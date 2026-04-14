use plotters::prelude::*;

use crate::knot_core::DirList;

// In pixels
const SQUARE_SIZE: u32 = 64;

const OUT_FILE_NAME: &str = "plotters-doc-data/sierpinski.png";
fn draw_to_file(
    vertlist: DirList,
    out_file_name: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = BitMapBackend::new(
        OUT_FILE_NAME,
        (
            SQUARE_SIZE * (vertlist.0.len() as u32),
            SQUARE_SIZE * (vertlist.0.len() as u32),
        ),
    )
    .into_drawing_area();

    root.fill(&WHITE)?;

    let root = root
        .titled("Sierpinski Carpet Demo", ("sans-serif", 60))?
        .shrink(((1024 - 700) / 2, 0), (700, 700));

    let text_style = ("sans-serif", 15.0).into_font().into_text_style(&root);
    for (i, (x, o)) in vertlist.0.into_iter().enumerate() {
        let _ = root.draw_text(
            "x",
            &text_style,
            ((SQUARE_SIZE * i as u32) as i32, (SQUARE_SIZE * x as u32) as i32),
        );

        let _ = root.draw_text(
            "o",
            &text_style,
            ((SQUARE_SIZE * i as u32) as i32, (SQUARE_SIZE * o as u32) as i32),
        );
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);

    Ok(())
}
