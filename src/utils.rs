use edgedb_tokio::Error;

pub fn print_edgedb_err(e: Error) -> Error {
    for (k, v) in e.headers() {
        let v = std::str::from_utf8(v.as_ref()).unwrap();
        println!("{k} -> {v}")
    }
    println!("kind_name => {}", e.kind_name());
    println!("kind_debug => {}", e.kind_debug());
    println!("initial_message => {:?}", e.initial_message());
    println!("hint => {:?}", e.hint());
    println!("details => {:?}", e.details());
    println!("position_start => {:?}", e.position_start());
    println!("position_end => {:?}", e.position_end());
    println!("line => {:?}", e.line());
    println!("column => {:?}", e.column());
    println!("code => {}", e.code());
    println!("server_traceback => {:?}", e.server_traceback());
    println!(
        "contexts => {:#?}",
        e.contexts().map(|x| format!("{x:?}")).collect::<Vec<_>>()
    );
    println!(
        "chain => {:#?}",
        e.chain().map(|x| format!("{x:?}")).collect::<Vec<_>>()
    );

    e

}