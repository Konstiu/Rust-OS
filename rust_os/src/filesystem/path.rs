use alloc::string::String;

use crate::filesystem::{Error, Result};

#[repr(transparent)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonPathString(String);

impl CanonPathString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for CanonPathString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl TryFrom<&str> for CanonPathString {
    type Error = Error;
    
    fn try_from(value: &str) -> Result<Self> {
        let canonicalized_str = canonicalize(value)?;
        Ok(CanonPathString(canonicalized_str))
    }
}

fn canonicalize(input: &str) -> Result<String> {
    if input.is_empty() || input == "." || input == "/" || input == "./" {
        return Ok(String::new());
    } 

    let mut s = input;
    while let Some(rest) = s.strip_prefix("./") {
        s = rest;
    }

    while let Some(rest) = s.strip_prefix('/') {
       s = rest; 
    }

    let mut out = String::with_capacity(s.len());
    let mut first = true;

    for comp in s.split('/') {
        if comp.is_empty() || comp == "." {
            continue;
        }
        if comp == ".." {
            return Err(Error::InvalidPathTraversal) 
        }

        if ! first {
           out.push('/'); 
        }

        first = false;
        out.push_str(comp);
    }

    Ok(out)
}