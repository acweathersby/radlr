mod llvm_test_parser;
pub use llvm_test_parser::*;

#[cfg(test)]
mod test
{
  #[link(name = "llvm_test_parser", kind = "static")]
  extern "C" {
    pub(crate) fn getUTF8CodePoint(input: *const u8) -> u32;
  }

  use crate::Context;
  use hctk::types::*;

  #[test]
  pub fn test_build()
  {
    let a = "\n    ";

    let cp_a = 0;
    // utf8::get_utf8_code_point_from(unsafe { *(a.as_bytes().as_ptr() as *const u32) });

    let cp_b = unsafe { getUTF8CodePoint(a.as_bytes().as_ptr()) };

    println!("a:{} b:{}", cp_a, cp_b);

    for action in Context::new_banner_parser(&mut UTF8StringReader::new("hello world")) {
      match action {
        ParseAction::Shift {
          skipped_characters: skip,
          token,
        } => {
          println!("Skip {:?} & Extract token {:?} ", skip, token);
        }
        ParseAction::Reduce {
          production_id,
          body_id,
          symbol_count,
        } => {
          println!(
            "Reduce {} symbols to production {} from completion of body {}",
            symbol_count, production_id, body_id,
          );
        }
        ParseAction::Accept { production_id } => {
          println!("Accept production {}", production_id);
          break;
        }
        _ => {
          break;
        }
      }
    }

    let actions = Context::new_banner_parser(&mut UTF8StringReader::new("hello world"))
      .collect::<Vec<_>>();

    assert!(matches!(actions[0], ParseAction::Shift { .. }));
    assert!(matches!(actions[1], ParseAction::Shift { .. }));
    assert!(
      matches!(actions[2], ParseAction::Reduce { production_id, .. } if production_id == 8)
    );
    assert!(
      matches!(actions[3], ParseAction::Accept { production_id } if production_id == 8)
    );
  }

  #[test]
  pub fn should_fail_on_second_erroneous_token()
  {
    let actions = Context::new_banner_parser(&mut UTF8StringReader::new("hello wold"))
      .collect::<Vec<_>>();
    println!("{:?}", actions);
    assert!(matches!(actions[0], ParseAction::Shift { .. }));
    assert!(matches!(actions[1], ParseAction::Error { .. }));
  }

  #[test]
  pub fn should_emit_end_of_input_action()
  {
    let mut reader = TestUTF8StringReader::new("hello world");

    reader.length = 5; // Artificially truncating the readers input window

    let actions = Context::new_banner_parser(&mut reader).collect::<Vec<_>>();

    assert!(matches!(actions[0], ParseAction::Shift { .. }));
    assert!(matches!(actions[1], ParseAction::EndOfInput { .. }));
    if let ParseAction::EndOfInput {
      current_cursor_offset,
    } = actions[1]
    {
      assert_eq!(current_cursor_offset, 5);
      println!("Offset position: {}", current_cursor_offset)
    }
  }
}
