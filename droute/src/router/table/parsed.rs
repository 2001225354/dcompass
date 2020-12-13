// Copyright 2020 LEXUGE
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[cfg(feature = "geoip")]
use super::rule::matchers::{GeoIpTarget, Geoip as MatGeoip};
use super::{
    rule::{
        actions::{Action, Disable as ActDisable, Query as ActQuery, Skip as ActSkip},
        matchers::{Any as MatAny, Domain as MatDomain, Matcher, QType as MatQType},
    },
    Result,
};
use crate::Label;
use hashbrown::HashSet;
#[cfg(feature = "serde-cfg")]
use serde::Deserialize;
use trust_dns_proto::rr::record_type::RecordType;

/// Actions to take
#[cfg_attr(feature = "serde-cfg", derive(Deserialize))]
#[cfg_attr(feature = "serde-cfg", serde(rename_all = "lowercase"))]
#[derive(Clone)]
pub enum ParsedAction {
    /// Set response to a message that "disables" requestor to retry.
    Disable,

    /// Do nothing on either response or query.
    Skip,

    /// Send query through an upstream with the specified tag name.
    Query(Label),
}

impl ParsedAction {
    // Should only be accessible from `Rule`.
    pub(super) fn convert(self) -> Box<dyn Action> {
        match self {
            Self::Disable => Box::new(ActDisable::default()),
            Self::Skip => Box::new(ActSkip::default()),
            Self::Query(t) => Box::new(ActQuery::new(t)),
        }
    }
}

#[cfg_attr(feature = "serde-cfg", derive(Deserialize))]
#[cfg_attr(feature = "serde-cfg", serde(rename_all = "lowercase"))]
#[derive(Clone, Eq, PartialEq)]
pub struct ParsedGeoIp {
    on: GeoIpTarget,
    codes: HashSet<String>,
}

/// Matchers to use
#[cfg_attr(feature = "serde-cfg", derive(Deserialize))]
#[cfg_attr(feature = "serde-cfg", serde(rename_all = "lowercase"))]
#[derive(Clone, Eq, PartialEq)]
pub enum ParsedMatcher {
    /// Matches any query
    Any,

    /// Matches domains in domain list files specified.
    Domain(Vec<String>),

    /// Matches query types provided. Query types are like AAAA, A, TXT.
    QType(HashSet<RecordType>),

    /// Matches if IP address in the record of the first response is in the list of countries. If specified, this can also match against source IP.
    #[cfg(feature = "geoip")]
    Geoip(ParsedGeoIp),
}

impl ParsedMatcher {
    // Should only be accessible from `Rule`.
    pub(super) async fn convert(self) -> Result<Box<dyn Matcher>> {
        Ok(match self {
            Self::Any => Box::new(MatAny::default()),
            Self::Domain(v) => Box::new(MatDomain::new(v).await?),
            Self::QType(types) => Box::new(MatQType::new(types)?),
            #[cfg(feature = "geoip")]
            Self::Geoip(s) => Box::new(MatGeoip::new(s.on, s.codes)?),
        })
    }
}

/// A rule composed of tag name, matcher, and branches.
#[cfg_attr(feature = "serde-cfg", derive(Deserialize))]
#[derive(Clone)]
pub struct ParsedRule {
    /// The tag name of the rule
    pub tag: Label,

    /// The matcher rule uses.
    #[cfg_attr(feature = "serde-cfg", serde(rename = "if"))]
    pub matcher: ParsedMatcher,

    /// If matcher matches, this branch specifies action and next rule name to route. Defaut to `(ParsedAction::Skip, "end".into())`
    #[cfg_attr(feature = "serde-cfg", serde(default = "default_branch"))]
    #[cfg_attr(feature = "serde-cfg", serde(rename = "then"))]
    pub on_match: (ParsedAction, Label),

    /// If matcher doesn't, this branch specifies action and next rule name to route. Defaut to `(ParsedAction::Skip, "end".into())`
    #[cfg_attr(feature = "serde-cfg", serde(default = "default_branch"))]
    #[cfg_attr(feature = "serde-cfg", serde(rename = "else"))]
    pub no_match: (ParsedAction, Label),
}

#[cfg(feature = "serde-cfg")]
fn default_branch() -> (ParsedAction, Label) {
    (ParsedAction::Skip, "end".into())
}
