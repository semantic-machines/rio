use bencher::{benchmark_group, benchmark_main, Bencher};
use chelone::Graph;
use rio_api::parser::TripleParser;
use rio_testsuite::manifest::TestManifest;
use rio_testsuite::parser_evaluator::{
    parse_w3c_rdf_test_file, read_w3c_rdf_test_file, TestEvaluationError,
};
use rio_turtle::{NTriplesParser, TurtleParser};
use serd_sys::{serd_reader_new, serd_reader_read_string, SerdSyntax};
use sophia::parser::nt;
use sophia::triple::stream::TripleSource;
use std::error::Error;
use std::ffi::CString;
use std::io::Read;
use std::path::PathBuf;

fn get_test_path() -> PathBuf {
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("rdf-tests");
    return base_path;
}

fn test_data_from_testsuite(
    manifest_uri: String,
    include_tests_types: &[&str],
) -> Result<Vec<u8>, Box<dyn Error>> {
    let test_path = get_test_path();
    let manifest = TestManifest::new(manifest_uri, |url| parse_w3c_rdf_test_file(url, &test_path));
    let mut data = Vec::default();
    for test in manifest {
        let test = test?;
        if include_tests_types.contains(&test.kind.iri.as_str()) {
            read_w3c_rdf_test_file(&test.action, &test_path)?
                .read_to_end(&mut data)
                .map_err(|e| TestEvaluationError::IO(test.action, e))?;
            data.push(b'\n');
        }
    }
    Ok(data)
}

fn ntriples_test_data() -> Result<Vec<u8>, Box<dyn Error>> {
    test_data_from_testsuite(
        "http://w3c.github.io/rdf-tests/ntriples/manifest.ttl".to_owned(),
        &["http://www.w3.org/ns/rdftest#TestNTriplesPositiveSyntax"],
    )
}

fn turtle_test_data() -> Result<Vec<u8>, Box<dyn Error>> {
    test_data_from_testsuite(
        "http://w3c.github.io/rdf-tests/turtle/manifest.ttl".to_owned(),
        &[
            "http://www.w3.org/ns/rdftest#TestTurtlePositiveSyntax",
            "http://www.w3.org/ns/rdftest#TestTurtleEval",
        ],
    )
}

fn parse_ntriples(bench: &mut Bencher, data: Vec<u8>) {
    bench.bytes = data.len() as u64;
    bench.iter(|| {
        let mut count: usize = 0;
        NTriplesParser::new(data.as_slice())
            .unwrap()
            .parse_all(&mut |_| {
                count += 1;
            })
    });
}

fn parse_turtle(bench: &mut Bencher, data: Vec<u8>) {
    bench.bytes = data.len() as u64;
    bench.iter(|| {
        let mut count: usize = 0;
        TurtleParser::new(data.as_slice(), "http://example.com/ex")
            .unwrap()
            .parse_all(&mut |_| {
                count += 1;
            })
    });
}

fn bench_parse_ntriples_with_ntriples(bench: &mut Bencher) {
    parse_ntriples(
        bench,
        match ntriples_test_data() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        },
    )
}

fn bench_parse_ntriples_with_turtle(bench: &mut Bencher) {
    parse_turtle(
        bench,
        match ntriples_test_data() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        },
    )
}

fn bench_parse_ntriples_with_serd(bench: &mut Bencher) {
    parse_with_serde(
        bench,
        match ntriples_test_data() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        },
        SerdSyntax::SERD_NTRIPLES,
    );
}

fn bench_parse_turtle_with_turtle(bench: &mut Bencher) {
    parse_turtle(
        bench,
        match turtle_test_data() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        },
    )
}

fn bench_parse_ntriples_with_sophia(bench: &mut Bencher) {
    let data = match ntriples_test_data() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    bench.bytes = data.len() as u64;
    bench.iter(|| nt::parse_read(data.as_slice()).in_sink(&mut ()).unwrap());
}

fn bench_parse_turtle_with_chelone(bench: &mut Bencher) {
    let data = String::from_utf8(match ntriples_test_data() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    })
    .unwrap();
    bench.bytes = data.len() as u64;
    bench.iter(|| {
        let graph = Graph::new(data.as_str()).unwrap();
        graph.parse();
    });
}

fn bench_parse_turtle_with_serd(bench: &mut Bencher) {
    parse_with_serde(
        bench,
        match ntriples_test_data() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{}", e);
                return;
            }
        },
        SerdSyntax::SERD_TURTLE,
    );
}

fn parse_with_serde(bench: &mut Bencher, mut data: Vec<u8>, format: SerdSyntax) {
    //Let's clean data
    for i in 0..data.len() {
        if data[i] == b'\0' {
            data[i] = b' ';
        }
    }

    bench.bytes = data.len() as u64;
    let data = CString::new(data).unwrap();
    bench.iter(|| unsafe {
        let mut count = 0;
        let reader = serd_reader_new(
            format,
            &mut count as *mut _ as *mut ::std::os::raw::c_void,
            None,
            None,
            None,
            None,
            None,
        );
        serd_reader_read_string(reader, data.as_bytes_with_nul().as_ptr());
    });
}

benchmark_group!(
    w3c_testsuite,
    bench_parse_ntriples_with_ntriples,
    bench_parse_ntriples_with_turtle,
    bench_parse_ntriples_with_serd,
    bench_parse_turtle_with_turtle,
    bench_parse_ntriples_with_sophia,
    bench_parse_turtle_with_chelone,
    bench_parse_turtle_with_serd
);
benchmark_main!(w3c_testsuite);
