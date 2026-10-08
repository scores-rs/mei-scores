//! The header (MEI.header): metadata about the encoded work, in
//! `<meiHead>`. Only the title statement is modeled.

/// `<meiHead>`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MeiHead {
    pub file_desc: FileDesc,
}

/// `<fileDesc>`: the bibliographic description of the file.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FileDesc {
    pub title_stmt: TitleStmt,
}

/// `<titleStmt>`: the work's title and who is responsible for it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TitleStmt {
    /// `<title>`'s text (several titles are joined).
    pub title: Option<String>,
    /// `<composer>`'s text. MEI 5 allows `<composer>` here directly; older
    /// files nest it in `<respStmt>`, which is read too.
    pub composer: Option<String>,
}
