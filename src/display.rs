use std::{cmp::{max, min}, fmt::Display, iter};

use crate::knot_core::{v_to_h, DirList};

impl Display for DirList {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horzlist = v_to_h(self);
        let mut downward_lines = vec![false; self.0.len()];
        for (x, o) in horzlist.0 {
            for i in 0..(self.0.len() as i32) {
                if i == min(x, o) {
                    print!(
                        "{}",
                        if downward_lines[min(x, o) as usize] {
                            "╰"
                        } else {
                            "╭"
                        }
                    );
                    downward_lines[i as usize] = !downward_lines[i as usize];
                } else if i == max(x, o) {
                    print!(
                        "{}",
                        if downward_lines[max(x, o) as usize] {
                            "╯"
                        } else {
                            "╮"
                        }
                    );
                    downward_lines[i as usize] = !downward_lines[i as usize];
                } else {
                    if downward_lines[i as usize] {
                        print!("│");
                    } else if min(x, o) < i && i < max(x, o) {
                        print!("─");
                    } else {
                        print!("·");
                    }
                }
            }
            println!();
        }
        std::fmt::Result::Ok(())
    }
}

impl std::fmt::Debug for DirList {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let horzlist = v_to_h(self);
        let hlen = horzlist.0.len();
        for (x, o) in &horzlist.0 {
            print!("{}", ".".repeat(*min(x, o) as usize));
            print!("{}", if x < o { "○" } else { "✗" });
            print!("{}", "·".repeat(((x - o).abs() - 1) as usize));
            print!("{}", if x < o { "✗" } else { "○" });
            print!(
                "{}",
                iter::repeat("·")
                    .take(hlen - (max(x, o) - 1) as usize)
                    .collect::<String>()
            );
            println!();
        }

        std::fmt::Result::Ok(())
    }
}
