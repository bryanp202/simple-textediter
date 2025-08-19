use std::fmt::Debug;
use std::iter::Iterator;

pub enum Rope {
    Branch {
        height: usize,
        weight: usize,
        line: usize,
        left: Box<Rope>,
        right: Box<Rope>,
    },
    Leaf (String)
}

impl Rope {
    const MAX_NODE_INSERT_SIZE: usize = 4096;
    pub fn new() -> Self {
        Rope::Leaf (String::new())
    }

    pub fn iter(&self) -> impl Iterator<Item=char> {
        RopeIterator::new(self)
    }

    pub fn len(&self) -> usize {
        match self {
            Rope::Branch { weight, right, .. } => weight + right.len(),
            Rope::Leaf (text) => text.chars().count(),
        }
    }
}

impl Rope {
    pub fn insert(self, index: usize, insert_text: &str) -> Self {
        let insert_text_len = insert_text.chars().count();
        let insert_text_lines = insert_text.lines().count() - 1;
        match self {
            Rope::Leaf (mut text) => {
                let text_len = text.chars().count();
                if index == text_len {
                    text.push_str(insert_text);
                    Rope::Leaf (text)
                } else if index <= text_len {
                    if text_len + text_len <= Self::MAX_NODE_INSERT_SIZE {
                        text.insert_str(index, insert_text);
                        Rope::Leaf(text)
                    } else {
                        let mut chars_iter = text.chars();
                        let mut new_left_str = String::with_capacity(index + insert_text_len);
                        new_left_str.extend(chars_iter.by_ref().take(index).chain(insert_text.chars()));
                        let mut new_right_str = String::with_capacity(text_len - index);
                        new_right_str.extend(chars_iter);
                        let right_text_lines = new_right_str.lines().count() - 1;
                        let line = text.lines().count() - 1;

                        Rope::Branch {
                            height: 1,
                            weight: index + insert_text_len,
                            line: line + insert_text_lines - right_text_lines,
                            left: Box::new(Rope::Leaf (new_left_str)),
                            right: Box::new(Rope::Leaf (new_right_str)),
                        }
                    }
                } else {
                    panic!("Out of bounds index: {}", index)
                }
            },
            Rope::Branch { height, weight, line, left, right } => {
                if index <= weight {
                    let left_weight = left.weight();
                    let left_branch = left.insert(index, insert_text);
                    let new_height = height.max(left_branch.height() + 1);

                    if (right.height() as isize - left_branch.height() as isize) < -1 && index <= left_weight {
                        Rope::Branch {
                            height: new_height,
                            weight: weight + insert_text_len,
                            line: line + insert_text_lines,
                            left: Box::new(left_branch),
                            right: right
                        }.rotate_right()
                    } else if (right.height() as isize - left_branch.height() as isize) < -1 && index > left_weight {
                        Rope::Branch {
                            height: new_height,
                            weight: weight + insert_text_len,
                            line: line + insert_text_lines,
                            left: Box::new(left_branch.rotate_left()),
                            right: right
                        }.rotate_right()
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
                    let right_branch = right.insert(index - weight, insert_text);
                    let new_height = height.max(right_branch.height() + 1);

                    if (left.height() as isize - right_branch.height() as isize) < -1 && index - weight > right_weight {
                        Rope::Branch {
                            height: new_height,
                            weight: weight,
                            line: line,
                            left: left,
                            right: Box::new(right_branch),
                        }.rotate_left()
                    } else if (left.height() as isize - right_branch.height() as isize) < -1 && index - weight <= right_weight {
                        Rope::Branch {
                            height: new_height,
                            weight: weight,
                            line: line,
                            left: left,
                            right: Box::new(right_branch.rotate_right()),
                        }.rotate_left()
                    } else {
                        Rope::Branch {
                            height: new_height,
                            weight: weight,
                            line: line,
                            left: left,
                            right: Box::new(right_branch),
                        }
                    }
                }
            },
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
                    weight: right_weight,
                    line: right_line,
                    left: right_left,
                    right: Box::new(Rope::Branch {
                        height: old_root_height,
                        weight: root_weight + right_weight,
                        line: root_line + right_line,
                        left: right_right,
                        right: root_left,
                    }),
                }
            } else { unreachable!() }
        } else { unreachable!() }
    }

    pub fn height(&self) -> usize {
        if let Rope::Branch { height, ..} = self {
            *height
        } else {
            0
        }
    }

    fn weight(&self) -> usize {
        match self {
            Rope::Branch { weight, .. } => *weight,
            Rope::Leaf (text) => text.chars().count()
        }
    }

    fn get_balance(&self) -> isize {
        if let Rope::Branch { left, right, ..} = self {
            left.height() as isize - right.height() as isize
        } else {
            0
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
            Rope::Leaf (text) => {
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
                if *idx < text.chars().count() {
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
                    Rope::Leaf (text) => {
                        self.current_leaf = Some((text, 0));
                        break;
                    },
                    Rope::Branch { right, .. } => self.push_left(right),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    /// Tests that a new rope is initialized as an empty string and the iterator can replicate that
    #[test]
    fn new_test() {
        let new_rope: String = Rope::new().iter().collect();
        assert_eq!(new_rope, "");
    }

    /// Inserts a string and checks output from iterator
    #[test]
    fn empty_insert_test() {
        let new_rope: String = Rope::new().insert(0, "Hello, world!").iter().collect();
        assert_eq!(new_rope, "Hello, world!");
    }

    /// Insert multiple string and checks output from iterator
    #[test]
    fn empty_multi_insert_test() {
        let new_rope: String = Rope::new()
            .insert(0, "Helloworld!")
            .insert(5, ", ")
            .iter().collect();
        assert_eq!(new_rope, "Hello, world!");
    }

    /// Insert multiple string and checks output from iterator and height
    #[test]
    fn empty_multi_insert_height_test() {
        let new_rope = Rope::new()
            .insert(0, "Helloworld!")
            .insert(5, ", ")
            .insert(0, "0")
            .insert(7, "0")
            .insert(2, "0")
            .insert(9, "0");
        let rope_str: String = new_rope.iter().collect();
        assert_eq!(rope_str, "0H0ello,00 world!");
        assert_eq!(new_rope.height(), 0)
    }

    /// Insert lots of random data at the end of a rope and assert iter output is the same
    #[test]
    fn empty_insert_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 10;
        let word_count = 10;
        for _ in 0..100 {
            let words: Vec<String> = (0..word_count)
                .map(|_| { (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect()})
                .collect();

            let correct_output: String = words.iter().map(|word| word.chars()).flatten().collect();

            let rope_output: String = words.iter()
                .fold(Rope::new(), |rope, word| {
                    let rope_len = rope.len();
                    rope.insert(rope_len, word)
                })
                .iter()
                .collect();

            assert_eq!(correct_output, rope_output);
        }
    }

    /// Insert lots of random data at random index a rope and assert iter output is the same
    #[test]
    fn empty_insert_random_fuzz_test() {
        let mut rng = rand::thread_rng();
        let word_len = 20;
        let word_count = 10000;
        for _ in 0..100 {
            let random_floats: Vec<f64> = (0..word_count)
                .map(|_| rng.gen_range(0.0..1.0))
                .collect();
            let words: Vec<String> = (0..word_count)
                .map(|_| { (0..word_len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect()})
                .collect();
            let words_copy = words.clone();

            let correct_output: String = random_floats.iter().zip(words_copy.into_iter())
                .fold(String::new(), |mut acc, (float, word)| {
                    acc.insert_str((acc.len() as f64 * float) as usize, &word);
                    acc
                });

            let rope_output: String = random_floats.into_iter().zip(words.into_iter())
                .fold(Rope::new(), |rope, (float, word)| {
                    let rope_len = rope.len() as f64;
                    rope.insert((rope_len * float) as usize, &word)
                })
                .iter()
                .collect();

            assert_eq!(correct_output, rope_output);
        }
    }
}