<> comment > line | block

<> line > "//" ( c:id | c:sp | c:sym | c:num )(*) c:nl

<> block > "/*"  ( c:id | c:sp | c:sym | c:num | c:nl )(*) "*/"