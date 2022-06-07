use crate::{
    grammar::{hash::hash_id_value, parse::ParseError},
    primitives::{
        Body, BodyId, BodySymbolRef, BodyTable, GrammarId, GrammarStore, Item,
        ProductionBodiesTable, ProductionId, ProductionTable, StringId, Symbol, SymbolID,
        SymbolStringTable, SymbolsTable, Token,
    },
};
use regex::Regex;

use super::{grammar_data::ast::Body as ASTBody, grammar_data::ast::*, parse};

use std::{
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    fs::read,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

type ImportProductionNameTable = HashMap<String, (String, PathBuf)>;

/// A temporary store of table references that can be passed
/// as one argument to functions that require access to such tables.
struct TempGrammarStore<'a> {
    /// Maps an imported symbol name to a universally unique string
    /// that may be used to resolve imported grammar production names.
    pub local_uuid: &'a String,
    pub absolute_path: &'a PathBuf,
    pub import_names_lookup: &'a mut ImportProductionNameTable,
    pub symbols_table: &'a mut SymbolsTable,
    pub symbols_string_table: &'a mut SymbolStringTable,
    pub bodies_table: &'a mut BodyTable,
    pub production_table: &'a mut ProductionTable,
    pub production_symbols_table: &'a mut BTreeSet<SymbolID>,
    pub production_bodies_table: &'a mut ProductionBodiesTable,
}

pub fn compile_all(
    root_grammar_absolute_path: &PathBuf,
) -> Result<GrammarStore, parse::ParseError> {
    let mut raw_grammars = Arc::new(Mutex::new(Vec::<GrammarStore>::new()));
    let mut claimed_grammar_paths = Arc::new(Mutex::new(HashSet::<PathBuf>::new()));
    let mut pending_grammar_paths = Arc::new(Mutex::new(VecDeque::<PathBuf>::new()));

    pending_grammar_paths
        .lock()
        .unwrap()
        .push_back(root_grammar_absolute_path.to_owned());

    let joins: Vec<JoinHandle<()>> = [0..4]
        .into_iter()
        .map(|_| {
            let mut claimed_grammar_paths = claimed_grammar_paths.clone();
            let mut raw_grammars = raw_grammars.clone();
            let mut pending_grammar_paths = pending_grammar_paths.clone();
            thread::spawn(move || {
                loop {
                    match {
                        let val = pending_grammar_paths.lock().unwrap().pop_front();
                        val
                    } {
                        Some(path) => {
                            if {
                                let result = claimed_grammar_paths
                                    .lock()
                                    .unwrap()
                                    .insert(path.to_owned());
                                result
                            } {
                                match compile_file_path(&path) {
                                    Ok(grammar) => {
                                        for (_, (_, b)) in &grammar.imports {
                                            pending_grammar_paths
                                                .lock()
                                                .unwrap()
                                                .push_back(b.to_owned());
                                        }
                                        raw_grammars.lock().unwrap().push(grammar);
                                    }
                                    Err(_) => {}
                                }
                            };
                        }
                        None => {
                            thread::sleep(Duration::from_nanos(250));
                            if {
                                let val = pending_grammar_paths.lock().unwrap().is_empty();
                                val
                            } && {
                                let val = claimed_grammar_paths.lock().unwrap().len()
                                    == raw_grammars.lock().unwrap().len();
                                val
                            } {
                                break;
                            }
                        }
                    }
                }
                return ();
            })
        })
        .collect();

    for join in joins {
        join.join();
    }

    let grammars = raw_grammars.lock().unwrap();
    let mut root = grammars[0].clone();

    //Merge grammars

    merge_grammars(&mut root, &grammars[1..]);

    //Create scanner productions

    create_scanner_productions(&mut root);

    //Update symbols

    process_symbols(&mut root);

    println!("{:#?}", root);

    Ok(root)
}

fn process_symbols(root: &mut GrammarStore) {
    let mut sym_id = SymbolID::DefinedSymbolIndexBasis;
    for (id, sym) in root.symbols_table.iter_mut() {
        if !sym.scanner_only {
            match id {
                SymbolID::TokenProduction(..)
                | SymbolID::DefinedGeneric(_)
                | SymbolID::DefinedNumeric(_)
                | SymbolID::DefinedIdentifier(_) => {
                    sym.index = sym_id;
                    sym_id += 1;
                }
                _ => {}
            }
        }
    }
}

fn create_scanner_productions(root: &mut GrammarStore) {
    // Start iterating over known token production references, and add new productions
    // as we encounter them.
    let mut scanner_production_queue = VecDeque::from_iter(root.symbols_table.keys().cloned());

    while let Some(sym_id) = scanner_production_queue.pop_front() {
        match &sym_id {
            SymbolID::DefinedGeneric(_)
            | SymbolID::DefinedNumeric(_)
            | SymbolID::DefinedIdentifier(_) => {
                let (_, scanner_production_id, scanner_name, symbol_string) =
                    get_scanner_info_from_defined(&sym_id, &*root);
                if !root.production_table.contains_key(&scanner_production_id) {
                    //Defined symbols are split along code points and individually packaged
                    let chars: Vec<char> = symbol_string.chars().collect();
                    let new_body_symbols: Vec<BodySymbolRef> = chars
                        .iter()
                        .enumerate()
                        .map(|(index, byte)| {
                            let string = byte.to_string();
                            let id = get_literal_id(&string);
                            if !root.symbols_table.contains_key(&id) {
                                root.symbols_string_table.insert(id, string);
                                root.symbols_table.insert(
                                    id,
                                    Symbol {
                                        byte_length: byte.len_utf8() as u32,
                                        code_point_length: 1,
                                        index: 0,
                                        uuid: id,
                                        scanner_only: true,
                                    },
                                );
                            }
                            BodySymbolRef {
                                annotation: String::default(),
                                consumable: true,
                                exclusive: false,
                                original_index: 0,
                                scanner_index: index as u32,
                                scanner_length: chars.len() as u32,
                                sym_id: id,
                            }
                        })
                        .collect();

                    // Insert new defined symbol derived data into root grammar.

                    let body_id = BodyId::new(&scanner_production_id, 0);

                    root.production_bodies_table
                        .insert(scanner_production_id, vec![body_id]);

                    root.bodies_table.insert(
                        body_id,
                        Body {
                            length: new_body_symbols.len() as u16,
                            symbols: new_body_symbols,
                            production: scanner_production_id,
                            id: body_id,
                        },
                    );

                    root.production_table.insert(
                        scanner_production_id,
                        crate::primitives::Production {
                            name: scanner_name,
                            id: scanner_production_id,
                            is_entry: false,
                            is_recursive: false,
                            is_scanner: true,
                            number_of_bodies: 1,
                            priority: 0,
                            token: Token::empty(),
                        },
                    );
                }
            }
            SymbolID::Production(prod_id, _) | SymbolID::TokenProduction(prod_id, _) => {
                let production = root.production_table.get(prod_id).unwrap().clone();
                let scanner_name = create_scanner_name(&production.name);
                let scanner_production_id = ProductionId::from(&scanner_name);

                if !root.production_table.contains_key(&scanner_production_id) {
                    let scanner_bodies: Vec<Body> = root
                        .production_bodies_table
                        .get(prod_id)
                        .unwrap()
                        .iter()
                        .enumerate()
                        .map(|(body_index, body_id)| {
                            let natural_body = root.bodies_table.get(body_id).unwrap();
                            let scanner_symbols = natural_body.symbols.iter().flat_map(|sym| {
                                let sym_id = &sym.sym_id;
                                match sym_id {
                                    // For any production or token production symbol encountered,
                                    // create a new symbol that references the equivalent scanner production
                                    // name, and submit this production for processing into a new scanner production.
                                    SymbolID::Production(_, grammar_id)
                                    | SymbolID::TokenProduction(_, grammar_id) => {
                                        let production =
                                            root.production_table.get(prod_id).unwrap();
                                        let scanner_name = create_scanner_name(&production.name);
                                        let scanner_production_id =
                                            ProductionId::from(&scanner_name);
                                        let new_symbol_id = SymbolID::Production(
                                            scanner_production_id,
                                            *grammar_id,
                                        );

                                        scanner_production_queue.push_back(*sym_id);

                                        vec![BodySymbolRef {
                                            annotation: String::default(),
                                            consumable: true,
                                            exclusive: sym.exclusive,
                                            original_index: 0,
                                            scanner_index: 0,
                                            scanner_length: 1,
                                            sym_id: new_symbol_id,
                                        }]
                                    }
                                    SymbolID::DefinedGeneric(_)
                                    | SymbolID::DefinedNumeric(_)
                                    | SymbolID::DefinedIdentifier(_) => {
                                        let (new_symbol_id, ..) =
                                            get_scanner_info_from_defined(sym_id, &*root);

                                        scanner_production_queue.push_back(*sym_id);

                                        vec![BodySymbolRef {
                                            annotation: String::default(),
                                            consumable: true,
                                            exclusive: sym.exclusive,
                                            original_index: 0,
                                            scanner_index: 0,
                                            scanner_length: 1,
                                            sym_id: new_symbol_id,
                                        }]
                                    }
                                    _ => vec![sym.clone()],
                                }
                            });

                            let symbols: Vec<BodySymbolRef> = scanner_symbols.collect();

                            Body {
                                id: BodyId::new(&scanner_production_id, body_index),
                                length: symbols.len() as u16,
                                production: scanner_production_id,
                                symbols,
                            }
                        })
                        .collect();

                    let mut bodies = vec![];

                    for body in scanner_bodies {
                        bodies.push(body.id);
                        root.bodies_table.insert(body.id, body);
                    }

                    root.production_table.insert(
                        scanner_production_id,
                        crate::primitives::Production {
                            name: scanner_name,
                            id: scanner_production_id,
                            is_entry: false,
                            is_recursive: false,
                            is_scanner: true,
                            number_of_bodies: bodies.len() as u16,
                            priority: 0,
                            token: production.token.clone(),
                        },
                    );

                    root.production_bodies_table
                        .insert(scanner_production_id, bodies);
                }
            }
            _ => {}
        }
    }
}

fn get_scanner_info_from_defined<'a>(
    sym_id: &SymbolID,
    root: &'a GrammarStore,
) -> (SymbolID, ProductionId, String, &'a String) {
    let symbol_string = root.symbols_string_table.get(&sym_id).unwrap();
    let scanner_name = create_scanner_name(symbol_string);
    let scanner_production_id = ProductionId::from(&scanner_name);
    let new_symbol_id = SymbolID::Production(scanner_production_id, root.uuid);
    (
        new_symbol_id,
        scanner_production_id,
        scanner_name,
        symbol_string,
    )
}

fn create_scanner_name(symbol_string: &String) -> String {
    format!("##scan__{}__", symbol_string)
}
#[test]
fn test_trivial_file_compilation() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test/compile/data/trivial.hcg");
    match compile_all(&path) {
        Err(err) => panic!("Failed! {:?}", err),
        Ok(_) => {}
    }
}

#[test]
fn test_trivial_file_compilation_with_single_import() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test/compile/data/trivial_importer.hcg");
    match compile_all(&path) {
        Err(err) => panic!("Failed! {}", err),
        Ok(grammar) => {
            for (id, ..) in grammar.bodies_table.iter() {
                let item = Item::from_body(id, &grammar).unwrap().increment().unwrap();

                println!("{}", item.debug_string(&grammar))
            }
        }
    }
}

///
/// Merge related grammars into a single GrammarStore
///
/// `root` is assumed to derived from the root source grammar, and grammars
/// are all other GrammarStores derived from grammars imported directly or
/// indirectly from the root source grammar.
fn merge_grammars(root: &mut GrammarStore, grammars: &[GrammarStore]) {
    let mut grammars_lookup = HashMap::<GrammarId, &GrammarStore>::new();

    // Merge grammar data into a single store
    for import_grammar in grammars {
        grammars_lookup.insert(import_grammar.uuid.clone(), import_grammar);

        // Merge all symbols
        for (id, sym) in &import_grammar.symbols_table {
            if !root.symbols_table.contains_key(id) {
                root.symbols_table.insert(id.clone(), sym.clone());
                match id {
                    SymbolID::DefinedGeneric(_)
                    | SymbolID::DefinedNumeric(_)
                    | SymbolID::DefinedIdentifier(_) => {
                        match import_grammar.symbols_string_table.get(id) {
                            Some(string) => {
                                root.symbols_string_table
                                    .insert(id.clone(), string.to_owned());
                            }
                            None => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Merge all referenced foreign productions into the root.
    let mut symbol_queue = VecDeque::from_iter(root.production_symbols_table.iter().cloned());

    while let Some(sym) = symbol_queue.pop_front() {
        if let Some(grammar_id) = sym.getGrammarId() {
            if grammar_id != root.uuid {
                match grammars_lookup.get(&grammar_id) {
                    Some(import_grammar) => {
                        if let Some(prod_id) = sym.getProductionId() {
                            if !root.production_table.contains_key(&prod_id) {
                                match import_grammar.production_table.get(&prod_id) {
                                    Some(production) => {
                                        //import the foreign production
                                        root.production_table
                                            .insert(prod_id.clone(), production.clone());

                                        let bodies = import_grammar
                                            .production_bodies_table
                                            .get(&prod_id)
                                            .unwrap()
                                            .clone();

                                        // Import all bodies referenced by this production
                                        for body_id in &bodies {
                                            let body = import_grammar
                                                .bodies_table
                                                .get(&body_id)
                                                .unwrap()
                                                .clone();

                                            //Add every Production symbol to the queue
                                            for sym in &body.symbols {
                                                match sym.sym_id {
                                                    SymbolID::Production(..) => {
                                                        symbol_queue.push_back(sym.sym_id.clone())
                                                    }
                                                    SymbolID::TokenProduction(prod, grammar) => {
                                                        if !root
                                                            .symbols_table
                                                            .contains_key(&sym.sym_id)
                                                        {
                                                            root.symbols_table.insert(
                                                                sym.sym_id,
                                                                import_grammar
                                                                    .symbols_table
                                                                    .get(&sym.sym_id)
                                                                    .unwrap()
                                                                    .clone(),
                                                            );
                                                        }

                                                        // Remap the production token symbol to regular a production symbol
                                                        // and submit as a merge candidate.
                                                        symbol_queue.push_back(
                                                            SymbolID::Production(prod, grammar),
                                                        )
                                                    }
                                                    _ => {}
                                                }
                                            }

                                            root.bodies_table.insert(body_id.clone(), body);
                                        }

                                        //Import the map of production id to bodies
                                        root.production_bodies_table
                                            .insert(prod_id.clone(), bodies);

                                        // Todo, remove this symbol from the production.
                                    }
                                    None => {
                                        panic!("Can't find production {}::{}", prod_id, grammar_id);
                                    }
                                }
                            }
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

pub fn compile_file_path(absolute_path: &PathBuf) -> Result<GrammarStore, parse::ParseError> {
    match read(absolute_path) {
        Ok(buffer) => {
            let grammar = parse::compile_ast(buffer)?;
            pre_process_grammar(&grammar, &String::from(""), absolute_path)
        }
        Err(err) => Err(parse::ParseError::IO_ERROR(err)),
    }
}

pub fn compile_string(
    string: &String,
    absolute_path: &PathBuf,
) -> Result<GrammarStore, parse::ParseError> {
    let grammar = parse::compile_ast(Vec::from(string.as_bytes()))?;
    pre_process_grammar(&grammar, string, absolute_path)
}

fn get_uuid_grammar_name(uri: &PathBuf) -> Result<String, ParseError> {
    match uri.file_name() {
        Some(name) => {
            let file_name = String::from(name.to_str().unwrap());

            let hash = unsafe {
                format!("{:x}", hash_id_value(&uri))
                    .get_unchecked(0..5)
                    .to_owned()
            };

            Ok(file_name + &hash)
        }
        None => Err(ParseError::UNDEFINED),
    }
}

/// Takes a Grammar produces core primitive tables;
///
/// ## Arguments
///
/// - `grammar` - A hcg AST node
/// - `source` - The source string of the hcg.
/// - `absolute_path` - The absolute path of the hcg's source file Used to resolve linked grammars.
///  
fn pre_process_grammar(
    grammar: &Grammar,
    source: &String,
    absolute_path: &PathBuf,
) -> Result<GrammarStore, parse::ParseError> {
    let mut import_names_lookup = ImportProductionNameTable::new();
    let mut production_bodies_table = ProductionBodiesTable::new();
    let mut production_table = ProductionTable::new();
    let mut bodies_table = BodyTable::new();
    let mut symbols_table = SymbolsTable::new();
    let mut symbols_string_table = SymbolStringTable::new();
    let mut post_process_productions: VecDeque<Box<Production>> = VecDeque::new();
    let mut production_symbols_table = BTreeSet::new();
    let uuid_name = get_uuid_grammar_name(&absolute_path)?;
    let uuid = GrammarId(hash_id_value(&uuid_name));

    {
        let mut tgs = TempGrammarStore {
            local_uuid: &uuid_name,
            absolute_path,
            import_names_lookup: &mut import_names_lookup,
            symbols_table: &mut symbols_table,
            symbols_string_table: &mut symbols_string_table,
            bodies_table: &mut bodies_table,
            production_table: &mut production_table,
            production_symbols_table: &mut production_symbols_table,
            production_bodies_table: &mut production_bodies_table,
        };

        // Process meta data, including EXPORT, IMPORT, and IGNORE meta data
        for obj in grammar.preamble.iter() {
            match obj {
                ASTNode::Ignore(ignore) => {}
                ASTNode::Import(import) => {
                    let mut uri = PathBuf::from(&import.uri);
                    let local_name = import.reference.String();

                    // Resolve path names. Since this touches the filesystem,
                    // it's bypassed when running tests to keep tests pure.

                    if !uri.is_absolute() {
                        match absolute_path.parent() {
                            None => {}
                            Some(new_path) => {
                                let mut new_path = new_path.to_owned();
                                new_path.push(uri);

                                match new_path.canonicalize() {
                                    Ok(result) => uri = result,
                                    Err(err) => {
                                        panic!(
                                    "\n{} \n{}",
                                    import
                                    .Token()
                                    .blame(1, 1, "Problem encountered when verifying import")
                                    .unwrap_or(
                                        String::from(
                                            "Problem encountered when verifying import: "
                                        ) + &import.uri
                                    ),
                                        err
                                    );
                                    }
                                }
                            }
                        }
                    }

                    let import_uuid = get_uuid_grammar_name(&uri)?;

                    // Map the foreign grammar's local name to the uuid and absolute path

                    tgs.import_names_lookup
                        .insert(local_name, (import_uuid, uri));
                }
                ASTNode::Export(export) => {}
                _ => {}
            }
        }
        // Process main grammar data, which include
        // Productions, IR states, and out of band functions
        for node in grammar.content.iter() {
            match node {
                ASTNode::Production(_) => {
                    pre_process_production(&node, &mut tgs, &mut post_process_productions)
                }
                ASTNode::ProductionMerged(prod) => {}
                ASTNode::IR_STATE(ir_state) => {}
                ASTNode::Out_Of_Band(oob_fn) => {}
                _ => {}
            }
        }

        // Continue processing any generated productions. This may loop
        // for a while as any given production may have several nested
        // anonymous productions through lists `...(+) | ...(*)` and
        // groups `(... | ...)`
        loop {
            match post_process_productions.pop_front() {
                Some(node) => pre_process_production(
                    &ASTNode::Production(node),
                    &mut tgs,
                    &mut post_process_productions,
                ),
                None => {
                    break;
                }
            }
        }
    }

    Ok(GrammarStore {
        source_path: absolute_path.clone(),
        uuid,
        uuid_name,
        production_bodies_table,
        production_table,
        bodies_table,
        symbols_table,
        symbols_string_table,
        production_symbols_table,
        imports: import_names_lookup,
    })
}

fn pre_process_production(
    production_node: &ASTNode,
    tgs: &mut TempGrammarStore,
    post_process_productions: &mut VecDeque<Box<Production>>,
) {
    let mut body_index = 0;

    if let ASTNode::Production(prod) = production_node {
        let production_id = get_production_id(production_node, tgs);
        let production_name = get_resolved_production_name(production_node, tgs);
        let mut bodies = vec![];

        match tgs.production_table.get(&production_id) {
            Some(existing_production) => {
                panic!(
                    "\n{}\n{}",
                    production_node
                        .Token()
                        .blame(
                            1,
                            1,
                            &format!("production {} already exists!", production_name)
                        )
                        .unwrap_or(format!("production {} already exists!", production_name)),
                    existing_production
                        .token
                        .blame(
                            1,
                            1,
                            &format!("production {} first defined here!", production_name)
                        )
                        .unwrap_or(format!(
                            "production {} first defined here!",
                            production_name
                        ))
                )
            }
            None => (),
        };

        // Extract body data and gather symbol information
        for body in &prod.bodies {
            if let ASTNode::Body(body) = body {
                let (new_bodies, productions) = pre_process_body(production_node, body, tgs);

                for prod in productions {
                    post_process_productions.push_back(prod);
                }

                for mut body in new_bodies {
                    let id = BodyId::new(&production_id, body_index);
                    body.id = id;
                    tgs.bodies_table.insert(id, body);
                    bodies.push(id);
                    body_index += 1;
                }
            }
        }

        tgs.production_table.insert(
            production_id,
            crate::primitives::Production {
                id: production_id,
                name: production_name,
                number_of_bodies: bodies.len() as u16,
                is_scanner: false,
                is_entry: false,
                is_recursive: false,
                priority: 0,
                token: production_node.Token(),
            },
        );

        tgs.production_bodies_table.insert(production_id, bodies);
    }
}

///
/// Get the resolved production of name applicable nodes.
/// Nodes from which a production name can be derived:
/// - Production_Symbol
/// - Production_Token
/// - Production
/// - Import_Production
///
/// ## Panics
/// This function panics if the node is not one of the above.
///
/// This function also panics if a local imported grammar name does not have
/// a matching `@IMPORT` statement.
fn get_resolved_production_name(node: &ASTNode, tgs: &TempGrammarStore) -> String {
    match node {
        ASTNode::Production_Import_Symbol(prod_imp_sym) => {
            let production_name = &prod_imp_sym.name;
            let local_import_grammar_name = &prod_imp_sym.module;

            match tgs.import_names_lookup.get(local_import_grammar_name) {
                None => {
                    panic!(
                    "\n{}",
                    node.Token()
                        .blame(
                            1,
                            1,
                            &format!("Unknown grammar : The local grammar name {} does not match any imported grammar names", local_import_grammar_name)
                        )
                        .unwrap_or(format!("Unknown grammar : The local grammar name {} does not match any imported grammar names", local_import_grammar_name))
                )
                }
                Some((grammar_uuid_name, _)) => {
                    grammar_uuid_name.to_owned() + "#" + production_name
                }
            }
        }
        ASTNode::Production_Symbol(prod_sym) => tgs.local_uuid.to_owned() + "#" + &prod_sym.name,
        ASTNode::Production(prod) => get_resolved_production_name(&prod.symbol, tgs),
        ASTNode::Production_Token(prod_tok) => {
            get_resolved_production_name(&prod_tok.production, tgs)
        }
        _ => {
            panic!(
                "\n{}",
                node.Token()
                    .blame(
                        1,
                        1,
                        "Unexpected node: Unable to resolve production name of this node!"
                    )
                    .unwrap_or(String::from(
                        "Unexpected node: Unable to resolve production name of this node!"
                    ))
            );
        }
    }
}

///
/// Get the resolved grammar data of applicable nodes.
/// Nodes from which a grammar name can be derived:
/// - Production_Symbol
/// - Production_Token
/// - Production
/// - Import_Production
///
/// ## Returns
/// A Tuple comprised of the grammar 0:uuid_name, 1:local_name, and 2:absolute_path.
/// local_name is `root` if the grammar maps to currently rendered grammar.
///
/// ## Panics
/// This function panics if the node is not one of the above.
///
/// This function also panics if a local imported grammar name does not have
/// a matching `@IMPORT` statement.
fn get_grammar_info_from_node<'a>(
    node: &'a ASTNode,
    tgs: &'a TempGrammarStore,
) -> (&'a str, &'a str, &'a PathBuf) {
    match node {
        ASTNode::Production_Import_Symbol(prod_imp_sym) => {
            let production_name = &prod_imp_sym.name;
            let local_import_grammar_name = &prod_imp_sym.module;
            match tgs.import_names_lookup.get(local_import_grammar_name) {
                None => {
                    panic!(
                    "\n{}",
                    node.Token()
                        .blame(
                            1,
                            1,
                            &format!("Unknown grammar : The local grammar name {} does not match any imported grammar names", local_import_grammar_name)
                        )
                        .unwrap_or(format!("Unknown grammar : The local grammar name {} does not match any imported grammar names", local_import_grammar_name))
                )
                }
                Some((resolved_grammar_name, path)) => {
                    (resolved_grammar_name, local_import_grammar_name, path)
                }
            }
        }
        ASTNode::Production_Symbol(prod_sym) => (&tgs.local_uuid, "root", &tgs.absolute_path),
        ASTNode::Production(prod) => get_grammar_info_from_node(&prod.symbol, tgs),
        ASTNode::Production_Token(prod_tok) => {
            get_grammar_info_from_node(&prod_tok.production, tgs)
        }
        _ => {
            panic!(
                "\n{}",
                node.Token()
                    .blame(
                        1,
                        1,
                        "Unexpected node: Unable to resolve production name of this node!"
                    )
                    .unwrap_or(String::from(
                        "Unexpected node: Unable to resolve production name of this node!"
                    ))
            );
        }
    }
}

fn get_production_hash_from_node(node: &ASTNode, tgs: &TempGrammarStore) -> u64 {
    let name = get_resolved_production_name(node, tgs);
    hash_id_value(name)
}

fn pre_process_body(
    production: &ASTNode,
    body: &ASTBody,
    tgs: &mut TempGrammarStore,
) -> (Vec<Body>, Vec<Box<Production>>) {
    if let ASTNode::Returned(ret) = &body.reduce_function {
        // Extract the function and insert into function table?
        println!("{:?}", ret);
    }

    let production_name = get_resolved_production_name(production, tgs);

    fn create_body_vectors(
        symbols: &Vec<ASTNode>,
        production_name: &String,
        mut index: u32,
        tgs: &mut TempGrammarStore,
    ) -> (Vec<Vec<BodySymbolRef>>, Vec<Box<Production>>) {
        let mut bodies = vec![];
        let mut productions = vec![];
        bodies.push(vec![]);

        for sym in symbols {
            let starting_bodies = bodies.len();

            fn create_production(
                name: &String,
                bodies: &Vec<ASTNode>,
                token: Token,
            ) -> (ASTNode, Box<Production>) {
                // Create a virtual production and symbol to go in its place
                let symbol = ASTNode::Production_Symbol(
                    super::grammar_data::ast::Production_Symbol::new(name.clone(), token.clone()),
                );

                let production = super::grammar_data::ast::Production::new(
                    false,
                    symbol.clone(),
                    bodies.clone(),
                    false,
                    token.clone(),
                );

                (symbol, production)
            }

            let SymbolData {
                annotation,
                is_list,
                is_group,
                is_optional,
                is_no_consume,
                is_meta,
                is_exclusive,
                sym_atom,
            } = get_symbol_details(sym);

            if let Some(mut sym) = sym_atom.to_owned() {
                let mut generated_symbol = ASTNode::NONE;
                if is_meta {
                    // Separate meta data symbols into it's own table that maps
                    // meta symbols to a body and its index.
                    index;
                    continue;
                }

                if is_group {
                    // Need to create new production that the virtual group production is bound
                    // to, add it to the list of currently considered productions, and replace
                    // this symbol with a production symbol pointing to the group.

                    // Except, if there are no functions within the production bodies
                    // we can simply inline the symbols into one or more alternate sets
                    // of bodies alongside the existing bodies.

                    if let ASTNode::Group_Production(group) = sym {
                        if annotation.is_empty()
                            && group.bodies.iter().all(|b| {
                                if let ASTNode::Body(body) = b {
                                    // The body does not have a reduce function.
                                    body.reduce_function.GetType() == 0
                                } else {
                                    false
                                }
                            })
                        {
                            // For each body in the group clone the existing body lists
                            // and process each list independently, inserting the new symbols
                            // into the existing bodies. We must make sure the indices are preserved
                            // since only the last symbol in each body can be bound to the index
                            // of the group production symbol.

                            let mut pending_bodies = vec![];

                            for body in &group.bodies {
                                if let ASTNode::Body(body) = body {
                                    let (mut new_bodies, mut new_productions) = create_body_vectors(
                                        &body.symbols,
                                        production_name,
                                        9999,
                                        tgs,
                                    );

                                    for body in new_bodies.iter_mut() {
                                        if let Some(last) = body.last_mut() {
                                            last.original_index = index;
                                        }
                                    }

                                    pending_bodies.append(&mut new_bodies);
                                    productions.append(&mut new_productions);
                                }
                            }

                            let mut new_bodies = vec![];

                            for pending_body in pending_bodies {
                                for body in &mut bodies {
                                    let mut new_body = body.clone();
                                    new_body.extend(pending_body.iter().cloned());
                                    new_bodies.push(new_body)
                                }
                            }

                            bodies = new_bodies;

                            index += 1;

                            // We do not to process the existing symbol as it is now
                            // replaced with it's component symbols,
                            // so we'll skip the rest of the loop
                            continue;
                        } else {
                            let (prod_sym, production) = create_production(
                                &(production_name.to_owned() + "_group_" + &index.to_string()),
                                &group.bodies,
                                group.tok.clone(),
                            );

                            productions.push(production);
                            generated_symbol = prod_sym;
                            sym = &generated_symbol;
                        }
                    } else {
                        panic!(
                            "\n{}",
                            sym.Token()
                                .blame(1, 1, "I don't know what to do with this!")
                                .unwrap_or(String::from("I don't know what to do with this!"))
                        )
                    }
                }

                if is_list {
                    // Create a new production that turns  A => a into A => a | A => A a
                    // produce a symbol id that points to that production

                    if let ASTNode::List_Production(list) = sym {
                        //Create new bodies that will be bound to the symbol.
                        let body_a = super::grammar_data::ast::Body::new(
                            false,
                            vec![list.symbols.clone()],
                            None,
                            ASTNode::NONE,
                        );

                        let mut body_b = body_a.clone();

                        match list.terminal_symbol {
                            ASTNode::NONE => {}
                            _ => {
                                body_b.symbols.insert(0, list.terminal_symbol.clone());
                            }
                        }

                        let (prod_sym, mut production) = create_production(
                            &(production_name.to_owned() + "_list_" + &index.to_string()),
                            &vec![ASTNode::Body(body_b), ASTNode::Body(body_a)],
                            list.tok.clone(),
                        );

                        // Add the production symbol to the front of body be to make
                        // the body left recursive
                        if let ASTNode::Body(body) = &mut production.bodies[0] {
                            body.symbols.insert(0, prod_sym.clone());
                        }

                        productions.push(production);
                        generated_symbol = prod_sym;
                        sym = &generated_symbol;
                    } else {
                        panic!(
                            "\n{}",
                            sym.Token()
                                .blame(1, 1, "I don't know what to do with this!")
                                .unwrap_or(String::from("I don't know what to do with this!"))
                        )
                    }
                }

                if is_optional {
                    // Need to create new bodies that contains all permutations of encountered
                    // symbols except for the currently considered symbol. This is achieved by
                    // duplicating all body vecs, then adding the current symbol to the original
                    // vecs, but not the duplicates.
                    for entry in bodies.clone() {
                        bodies.push(entry)
                    }
                }

                let id = intern_symbol(sym, tgs);

                for i in 0..starting_bodies {
                    bodies[i].push(BodySymbolRef {
                        original_index: index,
                        sym_id: id.clone(),
                        annotation: annotation.clone(),
                        consumable: !is_no_consume,
                        exclusive: is_exclusive,
                        scanner_index: 0,
                        scanner_length: 0,
                    });
                }

                index += 1;
            }
        }

        (bodies, productions)
    }

    let (bodies, productions) = create_body_vectors(&body.symbols, &production_name, 0, tgs);

    (
        bodies
            .iter()
            .map(|b| Body {
                symbols: b.clone(),
                length: b.len() as u16,
                production: get_production_id(production, tgs),
                id: BodyId::default(),
            })
            .collect(),
        productions,
    )
}

fn get_production_id(production: &ASTNode, tgs: &mut TempGrammarStore) -> ProductionId {
    let name = get_resolved_production_name(production, tgs);
    ProductionId(hash_id_value(name))
}

///
/// Returns an appropriate SymbolID::Defined* based on the input string
fn get_literal_id(string: &String) -> SymbolID {
    let identifier = Regex::new(r"[\w_-][\w\d_-]*$").unwrap();
    let number = Regex::new(r"\d+$").unwrap();

    if number.is_match(string) {
        SymbolID::DefinedNumeric(StringId::from(string))
    } else if identifier.is_match(string) {
        SymbolID::DefinedIdentifier(StringId::from(string))
    } else {
        SymbolID::DefinedGeneric(StringId::from(string))
    }
}

///
/// Adds a symbol to the symbol_table
fn intern_symbol(sym: &ASTNode /*, symbols_table, */, tgs: &mut TempGrammarStore) -> SymbolID {
    fn process_literal(string: &String, tgs: &mut TempGrammarStore) -> SymbolID {
        let mut id = get_literal_id(string);

        if !tgs.symbols_table.contains_key(&id) {
            tgs.symbols_string_table
                .insert(id.clone(), string.to_owned());
            let byte_length = string.bytes().len() as u32;
            let code_point_length = string.chars().count() as u32;

            tgs.symbols_table.insert(
                id.clone(),
                Symbol {
                    index: 0,
                    uuid: id.clone(),
                    byte_length,
                    code_point_length,
                    scanner_only: false,
                },
            );
        }

        id
    }

    fn get_production_hash_ids(
        node: &ASTNode,
        tgs: &mut TempGrammarStore,
    ) -> (ProductionId, GrammarId) {
        match node {
            ASTNode::Production_Symbol(_) | ASTNode::Production_Import_Symbol(_) => (
                ProductionId(get_production_hash_from_node(node, tgs)),
                GrammarId(hash_id_value(get_grammar_info_from_node(node, tgs).0)),
            ),
            _ => {
                panic!()
            }
        }
    }

    fn process_production(node: &ASTNode, tgs: &mut TempGrammarStore) -> SymbolID {
        let (production_id, grammar_id) = get_production_hash_ids(node, tgs);
        let id = SymbolID::Production(production_id, grammar_id);
        tgs.production_symbols_table.insert(id);
        id
    }

    fn process_token_production(node: &Production_Token, tgs: &mut TempGrammarStore) -> SymbolID {
        match process_production(&node.production, tgs) {
            SymbolID::Production(prod_id, grammar_id) => {
                let production_id = SymbolID::Production(prod_id, grammar_id);
                let token_production_id = SymbolID::TokenProduction(prod_id, grammar_id);
                if !tgs.symbols_table.contains_key(&token_production_id) {
                    tgs.symbols_table.insert(
                        token_production_id,
                        Symbol {
                            index: 0,
                            uuid: production_id,
                            byte_length: 0,
                            code_point_length: 0,
                            scanner_only: false,
                        },
                    );
                }
                token_production_id
            }
            _ => SymbolID::Undefined,
        }
    }

    match sym {
        ASTNode::Generated(gen) => match gen.val.as_str() {
            "sp" => SymbolID::GenericSpace,
            "tab" => SymbolID::GenericHorizontalTab,
            "nl" => SymbolID::GenericNewLine,
            "id" => SymbolID::GenericIdentifier,
            "num" => SymbolID::GenericNumber,
            "sym" => SymbolID::GenericSymbol,
            "ids" => SymbolID::GenericIdentifiers,
            "nums" => SymbolID::GenericNumbers,
            "syms" => SymbolID::GenericSymbols,
            _ => SymbolID::Undefined,
        },
        ASTNode::Exclusive_Literal(literal) => process_literal(&literal.val, tgs),
        ASTNode::Literal(literal) => process_literal(&literal.val, tgs),
        ASTNode::End_Of_File(_) => SymbolID::EndOfFile,
        ASTNode::Production_Symbol(_) | ASTNode::Production_Import_Symbol(_) => {
            process_production(sym, tgs)
        }
        ASTNode::Production_Token(token) => process_token_production(token, tgs),
        _ => {
            panic!(
                "Unexpected ASTNode while attempting to intern symbol \n{}",
                sym.Token()
                    .blame(0, 0, "found here")
                    .unwrap_or(String::new()),
            )
        }
    }
}

struct SymbolData<'a> {
    pub annotation: String,
    pub is_list: bool,
    pub is_group: bool,
    pub is_optional: bool,
    pub is_no_consume: bool,
    pub is_meta: bool,
    pub is_exclusive: bool,
    pub sym_atom: Option<&'a ASTNode>,
}

/// Get a flattened view of a symbol's immediate AST
fn get_symbol_details<'a>(mut sym: &'a ASTNode) -> SymbolData<'a> {
    let mut data = SymbolData {
        annotation: String::new(),
        is_list: false,
        is_group: false,
        is_optional: false,
        is_no_consume: false,
        is_meta: false,
        is_exclusive: false,
        sym_atom: None,
    };

    loop {
        match sym {
            ASTNode::AnnotatedSymbol(annotated) => {
                data.annotation = annotated.reference.val.to_owned();
                sym = &annotated.symbol;
            }
            ASTNode::OptionalSymbol(optional) => {
                data.is_optional = true;
                sym = &optional.symbol;
            }
            ASTNode::NonCaptureSymbol(non_cap) => {
                data.is_no_consume = true;
                sym = &non_cap.sym;
            }
            ASTNode::Exclude(_) | ASTNode::Look_Ignore(_) => {
                data.is_meta = true;
                break;
            }
            ASTNode::Group_Production(_) => {
                data.is_group = true;
                break;
            }
            ASTNode::List_Production(_) => {
                data.is_list = true;
                break;
            }
            ASTNode::Optional_List_Production(_) => {
                data.is_list = true;
                data.is_optional = true;
                break;
            }
            ASTNode::Exclusive_Literal(_) => {
                data.is_exclusive = true;
                break;
            }
            // This symbol types are "real" symbols, in as much
            // as they represent actual parsable entities which are
            // submitted to the bytecode compiler for evaluation
            ASTNode::Generated(_)
            | ASTNode::Literal(_)
            | ASTNode::Empty(_)
            | ASTNode::End_Of_File(_)
            | ASTNode::Production_Symbol(_)
            | ASTNode::Production_Token(_)
            | ASTNode::Production_Import_Symbol(_) => {
                break;
            }
            _ => {
                panic!("Unexpected ASTNode {}", sym.GetType())
            }
        }
    }
    data.sym_atom = Some(sym);
    data
}

#[test]
fn test_pre_process_grammar() {
    let grammar = String::from(
        "\n@IMPORT ./test/me/out.hcg as bob \n<> a > bob::test tk:p?^test a(+,) ( \\1234 | t:sp? ( sp | g:sym g:sp ) f:r { basalt } ) \\nto <> b > tk:p p ",
    );
    if let Ok(grammar) = parse::compile_ast(Vec::from(grammar.as_bytes())) {
        match pre_process_grammar(&grammar, &String::from("/grammar"), &PathBuf::from("/test")) {
            Ok(grammar) => {}
            Err(_) => {
                panic!("Failed to parse and produce an AST of '<> a > b'");
            }
        }
    } else {
        panic!("Failed to parse and produce an AST of '<> a > b'");
    }
}
