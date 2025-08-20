use std::fmt::Debug;
use std::iter::Iterator;
use std::cmp::Ordering;

pub enum Rope {
    Branch {
        height: usize,
        weight: usize,
        line: usize,
        left: Box<Rope>,
        right: Box<Rope>,
    },
    Leaf(String)
}

impl Rope {
    const MAX_NODE_INSERT_SIZE: usize = 4096;
    pub fn new() -> Self {
        Rope::Leaf(String::new())
    }

    pub fn new_branch(height: usize, weight: usize, line: usize, left: Box<Rope>, right: Box<Rope>) -> Self {
        Rope::Branch { height, weight, line, left, right }
    }

    pub fn chars(&self) -> RopeIterator {
        RopeIterator::new(self)
    }

    pub fn lines(&self) -> RopeLineIterator {
        RopeLineIterator::new(self)
    }

    pub fn len(&self) -> usize {
        match self {
            Rope::Branch { weight, right, .. } => weight + right.len(),
            Rope::Leaf(text) => text.chars().count(),
        }
    }

    pub fn line_count(&self) -> usize {
        match self {
            Rope::Branch { line, right, .. } => *line + right.line_count(),
            Rope::Leaf(text) => Self::get_line_count(&text),
        }
    }

    pub fn insert(self, index: usize, insert_text: &str) -> Self {
        self._insert(
            index,
            insert_text,
            insert_text.chars().count(),
            Self::get_line_count(insert_text),
        )
    }

    pub fn remove(self, index: usize, len: usize) -> Self {
        let (result, _) = self._remove(index, len);
        result.map_or_else(Rope::new, |x| *x)
    }
}

impl Rope {
    fn _insert(self, index: usize, insert_text: &str, insert_text_len: usize, insert_text_lines: usize) -> Self {
        match self {
            Rope::Leaf(text) => Self::_insert_leaf(index,  insert_text, insert_text_lines, text),
            Rope::Branch {
                height,
                weight,
                line,
                left,
                right
            } => Self::_insert_branch(index, insert_text, insert_text_len, insert_text_lines, height, weight, line, left, right),
        }
    }

    fn _insert_leaf(index: usize, insert_text: &str, insert_text_len: usize, mut text: String) -> Self {
        let text_len = text.chars().count();
        if index > text_len {
            panic!("Out of bounds index: Leaf of len {}, Index {}", text_len, index);
        }
        if text_len + insert_text_len <= Self::MAX_NODE_INSERT_SIZE {
            let char_index= Self::get_char_index(&text, text_len, index);
            text.insert_str(char_index, insert_text);
            Rope::Leaf(text)
        } else {
            let mut chars_iter = text.chars();
            let mut insert_chars_iter = insert_text.chars();
            let half_insert_len = insert_text_len / 2;

            let mut new_left_str = String::with_capacity(index + half_insert_len);
            new_left_str.extend(chars_iter.by_ref().take(index).chain(insert_chars_iter.by_ref().take(half_insert_len)));

            let mut new_right_str = String::with_capacity(text_len - index + insert_text_len - half_insert_len);
            new_right_str.extend(insert_chars_iter.chain(chars_iter));
            let lines = Self::get_line_count(&new_left_str);

            Rope::new_branch(
                1,
                index + half_insert_len,
                lines,
                Box::new(Rope::Leaf(new_left_str)),
                Box::new(Rope::Leaf(new_right_str)),
            )
        }
    }

    fn _insert_branch(
        index: usize,
        insert_text: &str,
        insert_text_len: usize,
        insert_text_lines: usize,
        height: usize,
        weight: usize,
        line: usize,
        left: Box<Self>,
        right: Box<Self>,
    ) -> Self {
        if index <= weight {
            let left_weight = left.weight();
            let left_branch = left._insert(index, insert_text, insert_text_len, insert_text_lines);
            let new_height = height.max(left_branch.height() + 1);

            if (right.height() as isize - left_branch.height() as isize) < -1 && index <= left_weight {
                Rope::new_branch(
                    new_height,
                    weight + insert_text_len,
                    line + insert_text_lines,
                    Box::new(left_branch),
                    right,
                ).rotate_right()
            } else if (right.height() as isize - left_branch.height() as isize) < -1 && index > left_weight {
                Rope::new_branch(
                    new_height,
                    weight + insert_text_len,
                    line + insert_text_lines,
                    Box::new(left_branch.rotate_left()),
                    right,
                ).rotate_right()
            } else {
                Rope::Branch {
                    height: new_height,
                    weight: weight + insert_text_len,
                    line: line + insert_text_lines,
                    left: Box::new(left_branch),
                    right: right
                }
            }
        } else {
            let right_weight = right.weight();
            let right_branch = right._insert(index - weight, insert_text, insert_text_len, insert_text_lines);
            let new_height = height.max(right_branch.height() + 1);

            if (left.height() as isize - right_branch.height() as isize) < -1 && index - weight > right_weight {
                Rope::new_branch(
                    new_height,
                    weight,
                    line,
                    left,
                    Box::new(right_branch),
                ).rotate_left()
            } else if (left.height() as isize - right_branch.height() as isize) < -1 && index - weight <= right_weight {
                Rope::new_branch(
                    new_height,
                    weight,
                    line,
                    left,
                    Box::new(right_branch.rotate_right()),
                ).rotate_left()
            } else {
                Rope::new_branch(
                    new_height,
                    weight,
                    line,
                    left,
                    Box::new(right_branch),
                )
            }
        }
    }

    fn _remove(self, index: usize, delete_len: usize) -> (Option<Box<Self>>, usize) {
        match self {
            Rope::Leaf(text) => Self::_remove_leaf(index, delete_len, text),
            Rope::Branch {
                height,
                weight,
                line,
                left,
                right
            } => Self::_remove_branch(index, delete_len, height, weight, line, left, right),
        }
    }

    fn _remove_leaf(index: usize, delete_len: usize, text: String) -> (Option<Box<Self>>, usize) {
        let text_len = text.chars().count();
        if index >= text_len {
            panic!("Out of bounds index: Leaf of len {}, Index {}", text_len, index);
        }
        let len_after_index = text_len - index;
        match len_after_index.cmp(&delete_len) {
            Ordering::Equal => if index == 0 {
                (None, 0)
            } else {
                (Some(Box::new(Rope::Leaf(text.chars().take(index).collect()))), 0)
            },
            Ordering::Greater => {
                let mut char_iter = text.chars();
                let mut new_str = String::with_capacity(text_len - delete_len);
                new_str.extend(char_iter.by_ref().take(index));
                new_str.extend(char_iter.skip(delete_len));
                (Some(Box::new(Rope::Leaf(new_str))), 0)
            },
            Ordering::Less => if index == 0 {
                (None, delete_len - text_len)
            } else {
                let mut char_iter = text.chars();
                let mut new_str = String::with_capacity(index);
                new_str.extend(char_iter.by_ref().take(index));
                (Some(Box::new(Rope::Leaf(new_str))), delete_len - len_after_index)
            }
        }
    }

    fn _remove_branch(index: usize, delete_len: usize, height: usize, weight: usize, line: usize, left: Box<Self>, right: Box<Self>) -> (Option<Box<Self>>, usize) {
        if index < weight {
            match left._remove(index, delete_len) {
                (Some(left_branch), 0) => (
                    Some(Box::new(
                        Rope::new_branch(
                            left_branch.height() + 1,
                            left_branch.weight(),
                            left_branch.line_count(),
                            left_branch,
                            right,
                        )
                    )),
                    0,
                ),
                (Some(left_branch), remaining_del) => {
                    match right._remove(0, remaining_del) {
                        (Some(right_branch), remaining_del_len) => (
                            Some(Box::new(Rope::new_branch(
                                left_branch.height() + 1,
                                left_branch.weight(),
                                left_branch.line_count(),
                                left_branch,
                                right_branch,
                            ))),
                            remaining_del_len,
                        ),
                        (_, remaining_del_len) => (Some(left_branch), remaining_del_len),
                    }
                },
                (None, 0) => (Some(right), 0),
                (None, remaining_del) => right._remove(0, remaining_del),
            }
        } else {
            match right._remove(index - weight, delete_len) {
                (Some(right_branch), remaining_del_len) => (
                    Some(Box::new(Self::new_branch(
                        height,
                        weight,
                        line,
                        left,
                        right_branch,
                    ))),
                    remaining_del_len,
                ),
                (None, remaining_del_len) => (Some(left), remaining_del_len),
            }
        }
    }

    fn rotate_right(self) -> Self {
        if let Rope::Branch { 
            height: _,
            weight: root_weight,
            line: root_line,
            left: root_left,
            right: root_right
        } = self {
            if let Rope::Branch {
                height: _,
                weight: left_weight,
                line: left_line,
                left: left_left,
                right: left_right
            } = *root_left {
                let old_root_height = root_right.height().max(left_right.height()) + 1;
                Rope::Branch {
                    height: old_root_height.max(left_left.height()) + 1,
                    weight: left_weight,
                    line: left_line,
                    left: left_left,
                    right: Box::new(Rope::Branch {
                        height: old_root_height,
                        weight: root_weight - left_weight,
                        line: root_line - left_line,
                        left: left_right,
                        right: root_right,
                    }),
                }
            } else { unreachable!() }
        } else { unreachable!() }
    }

    fn rotate_left(self) -> Self {
        if let Rope::Branch { 
            height: _,
            weight: root_weight,
            line: root_line,
            left: root_left,
            right: root_right
        } = self {
            if let Rope::Branch {
                height: _,
                weight: right_weight,
                line: right_line,
                left: right_left,
                right: right_right
            } = *root_right {
                let old_root_height = root_left.height().max(right_left.height()) + 1;
                Rope::Branch {
                    height: old_root_height.max(right_right.height()) + 1,
                    weight: root_weight + right_weight,
                    line: root_line + right_line,
                    left: Box::new(Rope::Branch {
                        height: old_root_height,
                        weight: root_weight,
                        line: right_line,
                        left: root_left,
                        right: right_left,
                    }),
                    right: right_right,
                }
            } else { unreachable!() }
        } else { unreachable!() }
    }

    fn height(&self) -> usize {
        if let Rope::Branch { height, ..} = self {
            *height
        } else {
            0
        }
    }

    fn weight(&self) -> usize {
        match self {
            Rope::Branch { weight, .. } => *weight,
            Rope::Leaf(text) => text.chars().count()
        }
    }

    #[allow(dead_code)]
    fn get_balance(&self) -> isize {
        if let Rope::Branch { left, right, ..} = self {
            left.height() as isize - right.height() as isize
        } else {
            0
        }
    }

    fn get_line_count(text: &str) -> usize {
        text.chars().filter(|&c| c == '\n').count()
    }

    fn get_char_index(text: &str, text_len: usize, index: usize) -> usize {
        match index {
            i if i == text_len  => text.len(),
            0 => 0,
            _ => text.char_indices().nth(index).expect("Should be impossible").0
        }
    }

    fn as_str(&self, as_str: &mut String, tabs: usize) {
        match self {
            Rope::Branch { height, weight, line, left, right } => {
                as_str.push_str("Branch [\n");
                as_str.push_str("-".repeat(tabs+1).as_str());
                as_str.push_str(format!("h: {}, w: {}, ln: {},\n",
                    height,
                    weight,
                    line,
                ).as_str());
                as_str.push_str("-".repeat(tabs+1).as_str());
                as_str.push_str("l: { ");
                left.as_str(as_str, tabs + 1);
                as_str.push_str(" }},\n");
                as_str.push_str("-".repeat(tabs+1).as_str());
                as_str.push_str("l: { ");
                right.as_str(as_str, tabs + 1);
                as_str.push_str(" }},\n");
                as_str.push_str("-".repeat(tabs).as_str());
                as_str.push(']');
            },
            Rope::Leaf(text) => {
                as_str.push_str(format!("Leaf [txt: {:?}]", text).as_str());
            }
        }
    }
}

impl Debug for Rope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut as_str = String::with_capacity(self.height() as usize);
        self.as_str(&mut as_str, 0);
        f.write_str(&as_str)
    }
}

pub struct RopeIterator<'a> {
    stack: Vec<&'a Rope>,
    current_leaf: Option<(&'a String, usize)>,
}

impl <'a> RopeIterator<'a> {
    fn new(root: &'a Rope) -> Self {
        let mut stack = Vec::new();
        let mut node = root;

        while let Rope::Branch { left, .. } = node {
            stack.push(node);
            node = left;
        }
        stack.push(node);

        Self {
            stack,
            current_leaf: None,
        }
    }
    
    fn push_left(&mut self, mut node: &'a Rope) {
        while let Rope::Branch { left, .. } = node {
            self.stack.push(node);
            node = left;
        }
        self.stack.push(node);
    }
}

impl <'a> Iterator for RopeIterator<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((text, ref mut idx)) = self.current_leaf {
                if *idx < text.len() {
                    let ch = text[*idx..].chars().next()?;
                    *idx += ch.len_utf8();
                    return Some(ch);
                } else {
                    self.current_leaf = None;
                }
            }

            loop {
                let node = self.stack.pop()?;
                match node {
                    Rope::Leaf(text) => {
                        self.current_leaf = Some((text, 0));
                        break;
                    },
                    Rope::Branch { right, .. } => self.push_left(right),
                }
            }
        }
    }
}

pub struct RopeLineIterator<'a> {
    stack: Vec<&'a Rope>,
    current_leaf: Option<(&'a String, usize)>,
}

impl <'a> RopeLineIterator<'a> {
    fn new(root: &'a Rope) -> Self {
        let mut stack = Vec::new();
        let mut node = root;

        while let Rope::Branch { left, .. } = node {
            stack.push(node);
            node = left;
        }
        stack.push(node);

        Self {
            stack,
            current_leaf: None,
        }
    }
    
    fn push_left(&mut self, mut node: &'a Rope) {
        while let Rope::Branch { left, .. } = node {
            self.stack.push(node);
            node = left;
        }
        self.stack.push(node);
    }
}

impl <'a> Iterator for RopeLineIterator<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        let mut line_str = String::new();
        loop {
            while let Some((text, ref mut index)) = self.current_leaf {
                if *index < text.len() {
                    let ch = text[*index..].chars().next()?;
                    *index += ch.len_utf8();
                    if ch == '\n' {
                        if let Some('\r') = text[*index..].chars().next() {
                            *index += '\r'.len_utf8();
                        }
                        return Some(line_str);
                    } else {
                        line_str.push(ch);
                    }
                } else {
                    self.current_leaf = None;
                }
            }

            loop {
                let node = self.stack.pop()?;
                match node {
                    Rope::Leaf(text) => {
                        self.current_leaf = Some((text, 0));
                        break;
                    },
                    Rope::Branch { right, .. } => self.push_left(right),
                }
            }
        }
    }

    fn nth(&mut self, mut n: usize) -> Option<Self::Item> {
        loop {
            if let Some((text, ref mut index)) = self.current_leaf {
                if *index < text.len() {
                    let nth_newline = text[*index..].char_indices()
                        .filter(|&(_, c)| c == '\n')
                        .scan((n, 0), |(acc, _), (index, _)| { *acc -= 1; Some((*acc, index)) })
                        .take(n)
                        .last();
                    if let Some((new_n, new_index)) = nth_newline {
                        *index = new_index + '\n'.len_utf8();
                        if new_n == 0 {
                            if let Some('\r') = text[*index..].chars().next() {
                                *index += '\r'.len_utf8();
                            }
                            break;
                        }
                        n = new_n;
                    }
                }
            }

            loop {
                let node = self.stack.pop()?;
                if let Some(Rope::Branch { line, .. }) = self.stack.last() {
                    if *line < n {
                        continue;
                    }
                }
                match node {
                    Rope::Leaf(text) => {
                        self.current_leaf = Some((text, 0));
                        break;
                    },
                    Rope::Branch { right, line, .. } => {
                        n -= line;
                        self.push_left(right);
                    }
                }
            }
        }

        self.next()
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use super::*;

    #[test]
    fn new_test() {
        let new_rope: String = Rope::new().chars().collect();
        assert_eq!(new_rope, "");
    }

    #[test]
    fn empty_insert_test() {
        let new_rope: String = Rope::new().insert(0, "Hello, world!").chars().collect();
        assert_eq!(new_rope, "Hello, world!");
    }

    #[test]
    fn multi_insert_test() {
        let new_rope: String = Rope::new()
            .insert(0, "Helloworld!")
            .insert(5, ", ")
            .chars().collect();
        assert_eq!(new_rope, "Hello, world!");
    }

    #[test]
    fn multi_insert_height_test() {
        let new_rope = Rope::new()
            .insert(0, "Helloworld!")
            .insert(5, ", ")
            .insert(0, "0")
            .insert(7, "0")
            .insert(2, "0")
            .insert(9, "0");
        assert_eq!(new_rope.chars().collect::<String>(), "0H0ello,00 world!");
        assert_eq!(new_rope.height(), 0)
    }

    #[test]
    fn insert_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 30;
        let word_count = 10_000;
        for _ in 0..100 {
            let words: Vec<String> = (0..word_count)
                .map(|_| (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect())
                .collect();

            let correct_output: String = words.iter().map(|word| word.chars()).flatten().collect();

            let rope_output: String = words.iter()
                .fold(Rope::new(), |rope, word| {
                    let rope_len = rope.len();
                    rope.insert(rope_len, word)
                })
                .chars()
                .collect();

            assert_eq!(correct_output, rope_output);
        }
    }

    #[test]
    fn insert_random_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 30;
        let word_count = 10_000;
        for _ in 0..100 {
            let random_floats: Vec<f64> = (0..word_count)
                .map(|_| rng.gen_range(0.0..1.0))
                .collect();
            let words: Vec<String> = (0..word_count)
                .map(|_| { (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect()})
                .collect();

            let correct_output: String = random_floats.iter().zip(words.iter())
                .fold(String::new(), |mut acc, (float, word)| {
                    acc.insert_str((acc.chars().count() as f64 * float) as usize, &word);
                    acc
                });

            let rope_output: String = random_floats.into_iter()
                .zip(words.iter())
                .fold(Rope::new(), |rope, (float, word)| {
                    let rope_len = rope.len() as f64;
                    rope.insert((rope_len * float) as usize, &word)
                })
                .chars()
                .collect();

            assert_eq!(correct_output.len(), rope_output.len());
            assert_eq!(correct_output, rope_output);
        }
    }

    #[test]
    fn rotate_right_test() {
        let rope = Rope::Branch {
            height: 2,
            weight: 7,
            line: 3,
            left: Box::new(Rope::Branch {
                height: 1,
                weight: 4,
                line: 1,
                left: Box::new(Rope::Leaf(String::from("TES\n"))),
                right: Box::new(Rope::Leaf(String::from("t\n\n"))),
            }),
            right: Box::new(Rope::Leaf(String::from("xin\n"))),
        };
        let rope = rope.rotate_right();
        assert_eq!(rope.height(), 2);
        assert_eq!(rope.len(), 11);
        assert_eq!(rope.line_count(), 4);
        assert_eq!(rope.weight(), 4);
        assert_eq!(rope.chars().collect::<String>(), String::from("TES\nt\n\nxin\n"));

        match &rope {
            Rope::Branch { left, right, .. } => { 
                assert_eq!(left.height(), 0);
                assert_eq!(left.len(), 4);
                assert_eq!(left.line_count(), 1);
                assert_eq!(left.weight(), 4);
                assert_eq!(left.chars().collect::<String>(), String::from("TES\n"));

                assert_eq!(right.height(), 1);
                assert_eq!(right.len(), 7);
                assert_eq!(right.line_count(), 3);
                assert_eq!(right.weight(), 3);
                assert_eq!(right.chars().collect::<String>(), String::from("t\n\nxin\n"));
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn rotate_left_test() {
        let rope = Rope::Branch {
            height: 2,
            weight: 4,
            line: 1,
            right: Box::new(Rope::Branch {
                height: 1,
                weight: 3,
                line: 1,
                left: Box::new(Rope::Leaf(String::from("ts\n"))),
                right: Box::new(Rope::Leaf(String::from("xin\n"))),
            }),
            left: Box::new(Rope::Leaf(String::from("TES\n"))),
        };

        let rope = rope.rotate_left();
        assert_eq!(rope.height(), 2);
        assert_eq!(rope.len(), 11);
        assert_eq!(rope.line_count(), 3);
        assert_eq!(rope.weight(), 7);
        assert_eq!(rope.chars().collect::<String>(), String::from("TES\nts\nxin\n"));

        match &rope {
            Rope::Branch { left, right, .. } => {
                assert_eq!(right.height(), 0);
                assert_eq!(right.len(), 4);
                assert_eq!(right.line_count(), 1);
                assert_eq!(right.weight(), 4);
                assert_eq!(right.chars().collect::<String>(), String::from("xin\n"));

                assert_eq!(left.chars().collect::<String>(), String::from("TES\nts\n"));
                assert_eq!(left.height(), 1);
                assert_eq!(left.line_count(), 2);
                assert_eq!(left.weight(), 4);
                assert_eq!(left.len(), 7);
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn rotate_right_left_test() {
        let rope = Rope::Branch {
            height: 2,
            weight: 7,
            line: 1,
            left: Box::new(Rope::Branch {
                height: 1,
                weight: 3,
                line: 1,
                left: Box::new(Rope::Leaf(String::from("ts\n"))),
                right: Box::new(Rope::Leaf(String::from("xing"))),
            }),
            right: Box::new(Rope::Leaf(String::from("TE\n\n"))),
        };
        let expected_height = rope.height();
        let expected_len = rope.len();
        let expected_weight = rope.weight();
        let expected_lines = rope.line_count();
        let expected_str: String = rope.chars().collect();
        let rope = rope.rotate_right().rotate_left();

        assert_eq!(rope.chars().collect::<String>(), expected_str);
        assert_eq!(rope.height(), expected_height);
        assert_eq!(rope.weight(), expected_weight);
        assert_eq!(rope.len(), expected_len);
        assert_eq!(rope.line_count(), expected_lines);
    }

    #[test]
    fn rotate_left_right_test() {
        let rope = Rope::Branch {
            height: 2,
            weight: 4,
            line: 0, 
            left: Box::new(Rope::Leaf(String::from("TEST"))),
            right: Box::new(Rope::Branch {
                height: 1,
                weight: 3,
                line: 0,
                left: Box::new(Rope::Leaf(String::from("tst"))),
                right: Box::new(Rope::Leaf(String::from("xing"))),
            }),
        };
        let expected_height = rope.height();
        let expected_len = rope.len();
        let expected_weight = rope.weight();
        let expected_str: String = rope.chars().collect();
        let expected_lines = rope.line_count();
        let rope = rope.rotate_left().rotate_right();

        assert_eq!(rope.chars().collect::<String>(), expected_str);
        assert_eq!(rope.height(), expected_height);
        assert_eq!(rope.weight(), expected_weight);
        assert_eq!(rope.len(), expected_len);
        assert_eq!(rope.line_count(), expected_lines);
    }

    #[test]
    fn non_ascii_insert_test() {
        let rope = Rope::new();
        let rope = rope.insert(0, "爆発しませんように");
        let rope = rope.insert(4, "何をしていますか？");

        assert_eq!(rope.chars().collect::<String>(), "爆発しま何をしていますか？せんように");
    }

    #[test]
    fn non_ascii_and_ascii_insert_test() {
        let rope = Rope::new();
        let rope = rope.insert(0, "爆発しませんように");
        let rope = rope.insert(4, "何をしていますか？");
        let rope = rope.insert(7, "hElLo!");

        assert_eq!(rope.chars().collect::<String>(), "爆発しま何をしhElLo!ていますか？せんように");
    }

    #[test]
    fn remove_test() {
        let mut rope = Rope::new();
        rope = rope.insert(0, "This is not cool!");
        rope = rope.remove(8, 4);
        assert_eq!(rope.chars().collect::<String>(), "This is cool!");
    }

    #[test]
    fn remove_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 30;
        let word_count = 100_000;
        for _ in 0..100 {
            let mut words: Vec<String> = (0..word_count)
                .map(|_| (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect())
                .collect();
            let word_to_add_remove = words.pop().unwrap();
            let word_to_add_remove_char_count = word_to_add_remove.chars().count();

            let correct_output: String = words.iter().map(|word| word.chars()).flatten().collect();

            let mut rope = words.iter().fold(Rope::new(), |rope, word| {
                let rope_len = rope.len();
                rope.insert(rope_len, word)
            });
            
            rope = rope.insert(100, &word_to_add_remove);
            rope = rope.remove(100, word_to_add_remove_char_count);
            assert_eq!(rope.chars().collect::<String>(), correct_output);
        }
    }

    #[test]
    fn remove_random_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 30;
        let word_count = 10_000;
        for _ in 0..100 {
            let mut random_floats: Vec<f64> = (0..word_count)
                .map(|_| rng.gen_range(0.0..1.0))
                .collect();
            let mut words: Vec<String> = (0..word_count)
                .map(|_| { (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect()})
                .collect();

            let insert_remove_ratio = random_floats.pop().unwrap();
            let insert_remove_word = words.pop().unwrap();
            let insert_remove_word_char_count = insert_remove_word.chars().count();

            let correct_output: String = random_floats.iter()
                .zip(words.iter())
                .fold(String::new(), |mut acc, (float, word)| {
                    acc.insert_str((acc.chars().count() as f64 * float) as usize, &word);
                    acc
                }
            );

            let mut rope = random_floats.into_iter()
                .zip(words.iter())
                .fold(Rope::new(), |rope, (float, word)| {
                    let rope_len = rope.len() as f64;
                    rope.insert((rope_len * float) as usize, &word)
                }
            );
            let insert_index = (rope.len() as f64 * insert_remove_ratio) as usize;
            rope = rope.insert(insert_index, &insert_remove_word);
            rope = rope.remove(insert_index, insert_remove_word_char_count);
            
            assert_eq!(rope.chars().collect::<String>(), correct_output);
        }
    }

    #[test]
    fn lines_iter_test() {
        let rope = Rope::new().insert(0, "Hello!\n\rHow are you?\nI hope you are\r good!\n");
        assert_eq!(rope.chars().collect::<String>(), "Hello!\n\rHow are you?\nI hope you are\r good!\n");
        let mut rope_lines_iter = rope.lines();
        assert_eq!(rope_lines_iter.next(), Some(String::from("Hello!")));
        assert_eq!(rope_lines_iter.next(), Some(String::from("How are you?")));
        assert_eq!(rope_lines_iter.next(), Some(String::from("I hope you are\r good!")));
        assert_eq!(rope_lines_iter.next(), None);
    }
}