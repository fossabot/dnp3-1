package dev.gridio.dnp3.codegen.render.modules

import dev.gridio.dnp3.codegen.model._
import dev.gridio.dnp3.codegen.render._

object RangedVariationModule extends Module {

  override def lines(implicit indent: Indentation) : Iterator[String] = {
      "use crate::app::parse::range::{RangedSequence, Range};".eol ++
      "use crate::app::gen::variations::fixed::*;".eol ++
      "use crate::app::gen::variations::gv::Variation;".eol ++
      "use crate::util::cursor::ReadCursor;".eol ++
      "use crate::app::parse::parser::ObjectParseError;".eol ++
      "use crate::app::parse::bytes::RangedBytesSequence;".eol ++
      "use crate::app::parse::bit::{BitSequence, DoubleBitSequence};".eol ++
      "use crate::util::logging::log_items;".eol ++
      space ++
      rangedVariationEnumDefinition ++
      space ++
      rangedVariationEnumImpl
  }

  private def rangedVariationEnumDefinition(implicit indent: Indentation) : Iterator[String] = {

    def getVarDefinition(v: Variation) : Iterator[String] = v match {
      case _ : SingleBitField => s"${v.name}(BitSequence<'a>),".eol
      case _ : DoubleBitField => s"${v.name}(DoubleBitSequence<'a>),".eol
      case _ : AnyVariation => s"${v.name},".eol
      case _ : FixedSize => s"${v.name}(RangedSequence<'a, ${v.name}>),".eol
      case _ : SizedByVariation if v.parent.isStaticGroup =>  {
        s"${v.parent.name}Var0,".eol ++
        s"${v.parent.name}VarX(u8, RangedBytesSequence<'a>),".eol
      }
    }

    "#[derive(Debug, PartialEq)]".eol ++
      bracket("pub enum RangedVariation<'a>") {
        variations.iterator.flatMap(getVarDefinition)
      }

  }

  private def rangedVariationEnumImpl(implicit indent: Indentation) : Iterator[String] = {

    def getNonReadVarDefinition(v: Variation) : String = v match {
      case _ : AnyVariation => ""
      case _ : FixedSize => "(RangedSequence::parse(range, cursor)?)"
    }

    def getReadVarDefinition(v: Variation) : String = v match {
      case _ : AnyVariation => ""
      case _ : FixedSize => "(RangedSequence::empty())"
    }

    def getNonReadMatcher(v: Variation): Iterator[String] = v match {
      case _ : SingleBitField =>  {
        s"Variation::${v.name} => Ok(RangedVariation::${v.name}(BitSequence::parse(range, cursor)?)),".eol
      }

      case _ : DoubleBitField =>  {
        s"Variation::${v.name} => Ok(RangedVariation::${v.name}(DoubleBitSequence::parse(range, cursor)?)),".eol
      }

      case v : SizedByVariation => {
        s"Variation::${v.parent.name}(0) => Err(ObjectParseError::ZeroLengthOctetData),".eol ++
        bracketComma(s"Variation::${v.parent.name}(x) =>") {
          s"Ok(RangedVariation::${v.parent.name}VarX(x, RangedBytesSequence::parse(x, range.get_start(), range.get_count(), cursor)?))".eol
        }
      }
      case _ => s"Variation::${v.name} => Ok(RangedVariation::${v.name}${getNonReadVarDefinition(v)}),".eol
    }

    def getReadMatcher(v: Variation): Iterator[String] = v match {
      case _ : SingleBitField =>  {
        s"Variation::${v.name} => Ok(RangedVariation::${v.name}(BitSequence::empty())),".eol
      }

      case _ : DoubleBitField =>  {
        s"Variation::${v.name} => Ok(RangedVariation::${v.name}(DoubleBitSequence::empty())),".eol
      }
      case _ : SizedByVariation => {
        s"Variation::${v.parent.name}(0) => Ok(RangedVariation::${v.parent.name}Var0),".eol
      }
      case _ => s"Variation::${v.name} => Ok(RangedVariation::${v.name}${getReadVarDefinition(v)}),".eol
    }

    def getLogMatcher(v: Variation): Iterator[String] = v match {
      case _ : AnyVariation => s"RangedVariation::${v.name} => {}".eol
      case _ : SizedByVariation => {
        s"RangedVariation::${v.parent.name}Var0 => {}".eol ++
          s"RangedVariation::${v.parent.name}VarX(_,_) => {}".eol
      }
      case _ => s"RangedVariation::${v.name}(seq) => log_items(level, seq.iter()),".eol
    }

    bracket("impl<'a> RangedVariation<'a>") {
      "#[rustfmt::skip]".eol ++
      bracket("pub fn parse_non_read(v: Variation, range: Range, cursor: &mut ReadCursor<'a>) -> Result<RangedVariation<'a>, ObjectParseError>") {
        bracket("match v") {
          variations.flatMap(getNonReadMatcher).iterator ++ "_ => Err(ObjectParseError::InvalidQualifierForVariation(v)),".eol
        }
      } ++ space ++
        bracket("pub fn parse_read(v: Variation) -> Result<RangedVariation<'a>, ObjectParseError>") {
          bracket("match v") {
            variations.flatMap(getReadMatcher).iterator ++ "_ => Err(ObjectParseError::InvalidQualifierForVariation(v)),".eol
          }
        } ++ space ++
        bracket("pub fn log(&self, level : log::Level)") {
          bracket("match self") {
            variations.flatMap(getLogMatcher).iterator
          }
        }
    }


  }

  def variations : List[Variation] = {
    ObjectGroup.allVariations.flatMap { v =>
      v match {
        case _ : DoubleBitField => Some(v)
        case _ : SingleBitField => Some(v)
        case v : AnyVariation if v.parent.isStaticGroup => Some(v)
        case v : FixedSize if v.parent.isStaticGroup => Some(v)
        case v : SizedByVariation if v.parent.isStaticGroup => Some(v)
        case _ => None
      }
    }
  }

}
