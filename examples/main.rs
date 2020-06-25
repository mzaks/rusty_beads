use rusty_beads::builder::{TypedBeadsBuilder, IndexedBeadsBuilder};
use rusty_beads::bead_type::{BeadTypeSet, BeadType};
use rusty_beads::sequence::{TypedBeads, IndexedBeads, DedupBeads};
use std::convert::TryFrom;
use std::fs;
use std::io;
use rusty_beads::converters::{csv_to_indexed_string_beads, beads_to_dedup_beads, string_beads_to_double_beads, CsvFirstLine, string_beads_to_int_beads};
use std::time::Instant;
use std::fs::File;
use std::io::Write;
use std::cell::RefCell;
use rusty_beads::csv::{CsvColumnBuilder, csv_to_beads, OwningDedupBeadsBuilder};

fn main() {
    convert_us_county();
    convert_tech_crunchcontinental_usa();
    convert_sacramentorealestatetransactions();
    convert_sacramentocrime_january2006();
    convert_runways();
    convert_proteins_253();
    convert_navaids();
    convert_external_gene_ids();
    convert_covid_19_world_cases_deaths_testing();
    convert_county_population();
    convert_chromosome_y();
    convert_airport_frequencies();
    // convert_airport_csv();
    convert_airport_csv2();
    // read_csv();
    // test_csv();
    // bit_set();
    //
    // run_numbers();
    // run_strings();
}

fn bit_set() {
    let mut builder = TypedBeadsBuilder::new(
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    ).ok().unwrap();

    builder.push_bool(true);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(true);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);
    println!("{:?}", buffer);

    let beads = TypedBeads::new(
        buffer.as_slice(),
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    ).ok().unwrap();
    println!("Number of elements: {}", beads.len());

    for bead in beads.iter() {
        println!("{}", bead.to_bool())
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode_with_types(&mut buffer);
    println!("{:?}", buffer);

    let beads = TypedBeads::new_types_included(
        buffer.as_slice()
    ).ok().unwrap();
    println!("Number of elements: {}", beads.len());

    let sym_beads = beads.symmetric().ok().unwrap();
    println!("Value at index {} is {}", 3, sym_beads.get(3).unwrap().to_bool());
}

fn run_numbers() {
    let types = BeadTypeSet::new(&[BeadType::Vlq, BeadType::VlqZ, BeadType::U8, BeadType::I8]);
    let mut builder = TypedBeadsBuilder::new(
        &types
    ).ok().unwrap();
    let data = [145, -145, 4502345, -4502345, 0, 3945873459];
    for v in data.iter() {
        builder.push_int(*v);
    }

    let mut out: Vec<u8>  = vec![];
    builder.encode(&mut out);
    println!("Data as beads: {:?}, len: {}", out, out.len());

    let beads = TypedBeads::new(out.as_slice(), &types).ok().unwrap();

    println!("Beads count: {}", beads.len());
    for b in beads.iter() {
        println!("{}", b.to_int());
    }
    let numbers:Vec<i64> = beads.iter().map(|b| i64::try_from(b.to_int()).unwrap()).collect();
    println!("Numbers: {:?}", numbers);

    let numbers:Vec<f64> = beads.iter().map(|b| b.to_float()).collect();
    println!("Numbers: {:?}", numbers);

    for b in beads.iter() {
        println!("{:?}", b.to_bytes());
    }
}

fn run_strings() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None]);
    let mut builder = TypedBeadsBuilder::new(
        &types
    ).ok().unwrap();
    let data: Vec<Option<&str>> = vec![Some("hello"), None, Some("this is something"), None, None, Some("special"), Some("")];
    for v in data.iter() {
        match v {
            Some(s) => builder.push_string(s),
            None => builder.push_none()
        };
    }

    let mut out: Vec<u8>  = vec![];
    builder.encode(&mut out);
    println!("Data as beads: {:?}, len: {}", out, out.len());

    let beads = TypedBeads::new(out.as_slice(), &types).ok().unwrap();

    println!("Beads count: {}", beads.len());
    for b in beads.iter() {
        println!("{:?}", String::try_from(b));
    }
    let strings:Vec<Option<String>> = beads.iter().map(|b| {
        String::try_from(b).ok()
    }).collect();
    println!("Strings: {:?}", strings);

    for b in beads.iter() {
        println!("{:?}", b.to_bytes());
    }
}

fn read_csv() {
    let filename = "examples/annual-enterprise-survey-2018-financial-year-provisional-csv.csv";
    let content = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    let mut buffer = vec![];
    let now = Instant::now();
    csv_to_indexed_string_beads(content.as_str(), CsvFirstLine::Include, &mut buffer).expect("Something went wrong converting CSV to beads");
    println!("buffer size {} in: {}", buffer.len(), now.elapsed().as_micros());

    let mut file = File::create("examples/test.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");

    let root = IndexedBeads::new(&buffer).expect("Could not read buffer as indexed beads");
    println!("Number of columns: {}", root.len());
    let mut dedup_root_builder = IndexedBeadsBuilder::new();
    let mut dvec = vec![];
    for index in 0..root.len() {
        let b1 = root.get(index).expect("Could not read first column");
        let mut dd1 = vec![];
        beads_to_dedup_beads(b1, &BeadTypeSet::new(&[BeadType::Utf8]), &mut dd1).expect("");
        dvec.push(dd1);
    }

    for d in dvec.iter() {
        dedup_root_builder.push(d.as_slice());
    }

    let mut dd1 = vec![];
    dedup_root_builder.encode(&mut dd1);

    let mut file = File::create("examples/test_dd.beads").expect("could not create file");
    file.write_all(&dd1).expect("could not write to file");

    let root = IndexedBeads::new(&dd1).expect("");
    let c1 = DedupBeads::new(root.get(3).expect(""));

    println!("{}", c1.len().expect(""));

    for index in 0..c1.len().expect("") {
        println!("{}: {:?}", index, String::from_utf8(c1.get(index).expect("")).expect(""));
    }

}

fn test_csv() {
    let mut rdr = csv::Reader::from_path("examples/annual-enterprise-survey-2018-financial-year-provisional-csv.csv").expect("");
    let buffer = fs::read("examples/test_dd.beads").expect("");
    let root = IndexedBeads::new(&buffer).expect("");
    for (row_index, row) in rdr.records().enumerate() {
        let r = row.expect("");
        for (column_index, item) in r.iter().enumerate() {
            println!("row: {}, column: {}", row_index, column_index);
            let column_bead = DedupBeads::new(root.get(column_index).expect(""));
            assert_eq!(item, String::from_utf8(column_bead.get(row_index+1).expect("")).expect(""));
        }
    }
}

fn convert_airport_csv() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/airports.csv").expect("");
    let mut buffer = vec![];
    csv_to_indexed_string_beads(csv.as_str(), CsvFirstLine::Exclude, &mut buffer).expect("Something went wrong converting CSV to beads");
    println!("Beads Buffer size: {}", buffer.len());

    let root = IndexedBeads::new(&buffer).expect("Could not read buffer as indexed beads");
    println!("Number of columns: {}", root.len());

    let c4 = root.get(4).expect("");
    let mut c4_ = vec![];
    string_beads_to_double_beads(c4, &BeadTypeSet::new(&[BeadType::F64]), 0.0, &mut c4_).expect("");
    println!("Size {} to {}", c4.len(), c4_.len());

    let c5 = root.get(5).expect("");
    let mut c5_ = vec![];
    string_beads_to_double_beads(c5, &BeadTypeSet::new(&[BeadType::F64]), 0.0, &mut c5_).expect("");
    println!("Size {} to {}", c5.len(), c5_.len());

    let c6 = root.get(6).expect("");
    let mut c6_ = vec![];
    string_beads_to_int_beads(c6, &BeadTypeSet::new(&[BeadType::I16, BeadType::None]), &mut c6_).expect("");
    println!("Size {} to {}", c6.len(), c6_.len());

    let c7 = root.get(7).expect("");
    let mut c7_ = vec![];
    beads_to_dedup_beads(c7, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c7_).expect("");
    println!("Size {} to {}", c7.len(), c7_.len());

    let c8 = root.get(8).expect("");
    let mut c8_ = vec![];
    beads_to_dedup_beads(c8, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c8_).expect("");
    println!("Size {} to {}", c8.len(), c8_.len());

    let c9 = root.get(9).expect("");
    let mut c9_ = vec![];
    beads_to_dedup_beads(c9, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c9_).expect("");
    println!("Size {} to {}", c9.len(), c9_.len());

    let c10 = root.get(10).expect("");
    let mut c10_ = vec![];
    beads_to_dedup_beads(c10, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c10_).expect("");
    println!("Size {} to {}", c10.len(), c10_.len());

    let c11 = root.get(11).expect("");
    let mut c11_ = vec![];
    beads_to_dedup_beads(c11, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c11_).expect("");
    println!("Size {} to {}", c11.len(), c11_.len());

    // let c12 = root.get(12).expect("");
    // let mut c12_ = vec![];
    // beads_to_dedup_beads(c12, &BeadTypeSet::new(&[BeadType::Utf8]), &mut c12_).expect("");
    // println!("Size {} to {}", c12.len(), c12_.len());

    let c13 = root.get(13).expect("");
    let mut c13_ = vec![];
    string_beads_to_sparse_string_beads(c13, &mut c13_);
    println!("Size {} to {}", c13.len(), c13_.len());

    let c15 = root.get(15).expect("");
    let mut c15_ = vec![];
    string_beads_to_sparse_string_beads(c15, &mut c15_);
    println!("Size {} to {}", c15.len(), c15_.len());

    let c16 = root.get(16).expect("");
    let mut c16_ = vec![];
    string_beads_to_sparse_string_beads(c16, &mut c16_);
    println!("Size {} to {}", c16.len(), c16_.len());

    let c17 = root.get(17).expect("");
    let mut c17_ = vec![];
    string_beads_to_sparse_string_beads(c17, &mut c17_);
    println!("Size {} to {}", c17.len(), c17_.len());


    let mut dedup_root_builder = IndexedBeadsBuilder::new();

    dedup_root_builder.push(root.get(0).expect("")); //1
    dedup_root_builder.push(root.get(1).expect("")); //2
    dedup_root_builder.push(root.get(2).expect("")); //2
    dedup_root_builder.push(root.get(3).expect("")); //4
    dedup_root_builder.push(c4_.as_slice());
    dedup_root_builder.push(c5_.as_slice());
    dedup_root_builder.push(c6_.as_slice());
    dedup_root_builder.push(c7_.as_slice());
    dedup_root_builder.push(c8_.as_slice());
    dedup_root_builder.push(c9_.as_slice());
    dedup_root_builder.push(c10_.as_slice());
    dedup_root_builder.push(c11_.as_slice());
    dedup_root_builder.push(root.get(12).expect(""));
    dedup_root_builder.push(c13_.as_slice());
    dedup_root_builder.push(root.get(14).expect(""));
    dedup_root_builder.push(c15_.as_slice());
    dedup_root_builder.push(c16_.as_slice());
    dedup_root_builder.push(c17_.as_slice());

    let mut dvec = vec![];
    dedup_root_builder.encode(&mut dvec);

    println!("Size: {}", dvec.len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/airports.beads").expect("could not create file");
    file.write_all(&dvec).expect("could not write to file");
}

fn convert_airport_csv2() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/airports.csv").expect("");
    println!("CSV file airports.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::utf8()),             // 2
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 3
        RefCell::new(CsvColumnBuilder::utf8()),             // 4
        RefCell::new(CsvColumnBuilder::f32(0.00001)),              // 5
        RefCell::new(CsvColumnBuilder::f32(0.00001)),              // 6

        // RefCell::new(CsvColumnBuilder::f64()),              // 5
        // RefCell::new(CsvColumnBuilder::f64()),              // 6
        RefCell::new(CsvColumnBuilder::optional_vlqz()),    // 7
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 8
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 9
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 10
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 11
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 12
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 13
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 14
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 15
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 16
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 17
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 18
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());

    let mut file = File::create("/Users/mzaks/dev/BeadsTest/airports.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");

    // let reader = CsvColumnReader::new(&buffer).expect("");
    // let column = reader.vlq(0).expect("");
    // for (index, b) in column.iter().enumerate() {
    //     println!("{} : {}", index, b.to_int());
    // }
    // let column = reader.dedup_utf8(0).expect("");
    // for index in 0..column.len().expect("") {
    //     println!("{} : {}", index, std::str::from_utf8(column.get(index).expect("").as_slice()).expect(""));
    // }
}

fn convert_airport_frequencies() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/airport-frequencies.csv").expect("");
    println!("CSV file airport-frequencies.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::vlq()),              // 2
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 3
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 4
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 5
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 6
        // RefCell::new(CsvColumnBuilder::f16_or_f32(0.001)),// 6
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/airport-frequencies.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}


fn convert_chromosome_y() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/chromosome_Y.csv").expect("");
    println!("CSV file chromosome_Y.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::vlq()),              // 2
        RefCell::new(CsvColumnBuilder::vlq()),              // 3
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 4
        RefCell::new(CsvColumnBuilder::vlq()),              // 5
        RefCell::new(CsvColumnBuilder::utf8()),       // 6
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 7
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 8
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 9
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Include, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/chromosome_Y.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_county_population() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/County_Population.csv").expect("");
    println!("CSV file County_Population.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::utf8()),              // 1
        RefCell::new(CsvColumnBuilder::vlq()),              // 2
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 3
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 4
        RefCell::new(CsvColumnBuilder::vlq()),              // 5
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/County_Population.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_covid_19_world_cases_deaths_testing() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/covid-19-world-cases-deaths-testing.csv").expect("");
    println!("CSV file covid-19-world-cases-deaths-testing.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 1
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 2
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 3
        RefCell::new(CsvColumnBuilder::vlq()),              // 4
        RefCell::new(CsvColumnBuilder::vlqz()),             // 5
        RefCell::new(CsvColumnBuilder::vlq()),              // 6
        RefCell::new(CsvColumnBuilder::vlq()),              // 7
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 8
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 9
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 10
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 11
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 12
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 13
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 14
        RefCell::new(CsvColumnBuilder::optional_utf8()),     // 15
        RefCell::new(CsvColumnBuilder::optional_utf8()),    // 16
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/covid-19-world-cases-deaths-testing.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_external_gene_ids() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/external_gene_ids.csv").expect("");
    println!("CSV file external_gene_ids.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 2
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 3
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 4
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 5
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 6
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 7
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/external_gene_ids.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_navaids() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/navaids.csv").expect("");
    println!("CSV file navaids.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::utf8()),             // 2
        RefCell::new(CsvColumnBuilder::utf8()),             // 3
        RefCell::new(CsvColumnBuilder::utf8()),             // 4
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 5
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 6
        RefCell::new(CsvColumnBuilder::f64()),              // 7
        RefCell::new(CsvColumnBuilder::f64()),              // 8
        RefCell::new(CsvColumnBuilder::optional_vlqz()),    // 9
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 10
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 11
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 12
        RefCell::new(CsvColumnBuilder::optional_f64()),     // 13
        RefCell::new(CsvColumnBuilder::optional_f64()),     // 14
        RefCell::new(CsvColumnBuilder::optional_vlqz()),    // 15
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 16
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 17
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 18
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 19
        RefCell::new(CsvColumnBuilder::utf8()),             // 20
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/navaids.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_proteins_253() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/proteins_253.csv").expect("");
    println!("CSV file proteins_253.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),              // 1
        RefCell::new(CsvColumnBuilder::utf8()),             // 2
        RefCell::new(CsvColumnBuilder::utf8()),             // 3
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 4
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 5
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 6
        RefCell::new(CsvColumnBuilder::dedup_utf8()),       // 7
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Include, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/proteins_253.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_runways() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/runways.csv").expect("");
    println!("CSV file runways.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::vlq()),                  // 1    id
        RefCell::new(CsvColumnBuilder::vlq()),                  // 2    airport_ref
        RefCell::new(CsvColumnBuilder::utf8()),                 // 3    airport_ident
        RefCell::new(CsvColumnBuilder::optional_vlq()),         // 4    length_ft
        RefCell::new(CsvColumnBuilder::optional_vlq()),         // 5    width_ft
        RefCell::new(CsvColumnBuilder::dedup_utf8()),           // 6    surface
        RefCell::new(CsvColumnBuilder::bool("0", "1")),                   // 7    lighted
        RefCell::new(CsvColumnBuilder::bool("0", "1")),                   // 8    closed
        RefCell::new(CsvColumnBuilder::dedup_utf8()),           // 9    le_ident
        RefCell::new(CsvColumnBuilder::optional_f32(0.00001)),         // 10   le_latitude_deg
        RefCell::new(CsvColumnBuilder::optional_f32(0.00001)),         // 11   le_longitude_deg
        RefCell::new(CsvColumnBuilder::optional_f32(0.0)),         // 12   le_elevation_ft
        RefCell::new(CsvColumnBuilder::optional_f32(0.001)),         // 13   le_heading_degT
        RefCell::new(CsvColumnBuilder::optional_vlq()),         // 14   le_displaced_threshold_ft
        RefCell::new(CsvColumnBuilder::dedup_utf8()),           // 15   he_ident
        RefCell::new(CsvColumnBuilder::optional_f32(0.00001)),         // 16   he_latitude_deg
        RefCell::new(CsvColumnBuilder::optional_f32(0.00001)),         // 17   he_longitude_deg
        RefCell::new(CsvColumnBuilder::optional_f32(0.00001)),         // 18   he_elevation_ft
        RefCell::new(CsvColumnBuilder::optional_f32(0.001)),         // 19   he_heading_degT
        RefCell::new(CsvColumnBuilder::optional_vlq()),         // 20   he_displaced_threshold_ft
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/runways.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_sacramentocrime_january2006() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/SacramentocrimeJanuary2006.csv").expect("");
    println!("CSV file SacramentocrimeJanuary2006.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 1   cdatetime
        RefCell::new(CsvColumnBuilder::utf8()),                         // 2   address
        RefCell::new(CsvColumnBuilder::u8()),                           // 3   district
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 4   beat
        RefCell::new(CsvColumnBuilder::vlq()),                          // 5   grid
        RefCell::new(CsvColumnBuilder::utf8()),                         // 6   crimedescr
        RefCell::new(CsvColumnBuilder::vlq()),                          // 7   ucr_ncic_code
        RefCell::new(CsvColumnBuilder::f32(0.00001)),          // 8   latitude
        RefCell::new(CsvColumnBuilder::f32(0.00001)),          // 9   longitude
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/SacramentocrimeJanuary2006.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_sacramentorealestatetransactions() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/Sacramentorealestatetransactions.csv").expect("");
    println!("CSV file Sacramentorealestatetransactions.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::utf8()),                         // 1   street
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 2   city
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 3   zip
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 4   state
        RefCell::new(CsvColumnBuilder::vlq()),                          // 5   beds
        RefCell::new(CsvColumnBuilder::vlq()),                          // 5   baths
        RefCell::new(CsvColumnBuilder::vlq()),                          // 5   sq_ft
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 6   type
        RefCell::new(CsvColumnBuilder::dedup_utf8()),                   // 7   sale_date
        RefCell::new(CsvColumnBuilder::vlq()),                          // 5   price
        RefCell::new(CsvColumnBuilder::f32(0.00001)),          // 8   latitude
        RefCell::new(CsvColumnBuilder::f32(0.00001)),          // 9   longitude
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/Sacramentorealestatetransactions.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_tech_crunchcontinental_usa() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/TechCrunchcontinentalUSA.csv").expect("");
    println!("CSV file TechCrunchcontinentalUSA.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 1   permalink
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 2   company
        RefCell::new(CsvColumnBuilder::optional_vlq()),        // 3   numEmps
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 4   category
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 5   city
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 6   state
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 7   fundedData
        RefCell::new(CsvColumnBuilder::vlq()),                 // 8   raisedAmt
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 9   raisedCurrency
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 10  round
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/TechCrunchcontinentalUSA.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

fn convert_us_county() {
    let csv = fs::read_to_string("/Users/mzaks/dev/BeadsTest/us_county.csv").expect("");
    println!("CSV file us_county.csv size: {}", csv.as_bytes().len());
    let column_builders = vec![
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 1   date
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 2   county
        RefCell::new(CsvColumnBuilder::dedup_utf8()),          // 3   state
        RefCell::new(CsvColumnBuilder::optional_vlq()),        // 4   fips
        RefCell::new(CsvColumnBuilder::vlq()),                 // 5   cases
        RefCell::new(CsvColumnBuilder::vlq()),                 // 6   deaths
    ];
    let mut buffer = vec![];
    csv_to_beads(csv.as_str(), CsvFirstLine::Exclude, column_builders, &mut buffer).expect("");
    println!("Beads size is: {} of CSV: {}/{}", buffer.len() as f64 / csv.as_bytes().len() as f64, buffer.len(), csv.as_bytes().len());
    let mut file = File::create("/Users/mzaks/dev/BeadsTest/us_county.beads").expect("could not create file");
    file.write_all(&buffer).expect("could not write to file");
}

struct CsvColumnReader<'a> {
    root: IndexedBeads<'a>
}

impl <'a> CsvColumnReader<'a> {
    fn new(buffer: &'a[u8]) -> Result<CsvColumnReader, String> {
        let root: IndexedBeads<'a> = IndexedBeads::new(buffer)?;
        Ok(CsvColumnReader{
            root
        })
    }

    fn utf8(&self, index: usize) -> Result<TypedBeads, String> {
        Ok(TypedBeads::new(self.root.get(index)?, &BeadTypeSet::utf8())?)
    }
    fn optional_utf8(&self, index: usize) -> Result<TypedBeads, String> {
        Ok(TypedBeads::new(self.root.get(index)?, &BeadTypeSet::optional_utf8())?)
    }
    fn dedup_utf8(&self, index: usize) -> Result<DedupBeads, String> {
        Ok(DedupBeads::new(self.root.get(index)?))
    }
    fn f32(&self, index: usize) -> Result<TypedBeads, String> {
        Ok(TypedBeads::new(self.root.get(index)?, &BeadTypeSet::f32())?)
    }
    fn f64(&self, index: usize) -> Result<TypedBeads, String> {
        Ok(TypedBeads::new(self.root.get(index)?, &BeadTypeSet::f64())?)
    }
    fn vlq(&self, index: usize) -> Result<TypedBeads, String> {
        Ok(TypedBeads::new(self.root.get(index)?, &BeadTypeSet::vlq())?)
    }
}


fn string_beads_to_sparse_string_beads<W>(buffer: &[u8], writer: &mut W) where W: io::Write {
    let source = TypedBeads::new(buffer, &BeadTypeSet::utf8()).expect("");
    let mut builder = TypedBeadsBuilder::new(&BeadTypeSet::optional_utf8()).expect("");
    for b in source.iter() {
        let str = b.to_str();
        if str.is_empty() {
            builder.push_none();
        } else {
            builder.push_string(str);
        }
    }
    builder.encode(writer);
}