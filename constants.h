// -*- C++ -*-
//
// $Id: constants.h,v 1.7 2003/02/18 05:08:56 deven Exp $
//
// Global constants header file.
//
// Copyright 1992-1996, 2000-2003 by Deven T. Corzine.  All rights reserved.
//
// This file is part of the Gangplank conferencing system.
//
// This file may be distributed under the terms of the Q Public License
// as defined by Trolltech AS of Norway (except for Choice of Law) and as
// appearing in the file LICENSE.QPL included in the packaging of this file.
//
// This file is provided AS IS with NO WARRANTY OF ANY KIND, INCLUDING THE
// WARRANTY OF DESIGN, MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE.
//
// Visit <http://www.gangplank.org/license/> or contact <info@gangplank.org>
// for more information or if any conditions of this licensing are unclear.
//
// $Log: constants.h,v $
// Revision 1.7  2003/02/18 05:08:56  deven
// Updated copyright dates.
//
// Revision 1.6  2003/02/17 06:25:14  deven
// Removed DefaultPort constant in favor of using configured PORT parameter.
//
// Revision 1.5  2003/02/17 06:22:29  deven
// Moved InputSize constant into Telnet class, increased size to 1024 bytes.
//
// Revision 1.4  2003/02/17 06:20:16  deven
// Moved BlockSize constant into Block class, increased size to 4096 bytes.
//
// Revision 1.3  2003/02/17 06:16:20  deven
// Moved BufSize constant into Telnet class.
//
// Revision 1.2  2002/09/17 06:20:06  deven
// Modified DefaultPort to use PORT value from configure script.
//
// Revision 1.1  2001/11/30 23:53:32  deven
// Initial revision
//

// Check if previously included.
#ifndef _CONSTANTS_H
#define _CONSTANTS_H 1

// Internal character constants.
const unsigned char UnquotedUnderscore = 128, Separator = 129;

// ASCII character constants.
const unsigned char    Null         = 0,   ControlA    = 1,   ControlB    = 2,
   ControlC    = 3,    ControlD     = 4,   ControlE    = 5,   ControlF    = 6,
   ControlG    = 7,    Bell         = 7,   ControlH    = 8,   Backspace   = 8,
   ControlI    = 9,    Tab          = 9,   ControlJ    = 10,  Linefeed    = 10,
   Newline     = 10,   ControlK     = 11,  ControlL    = 12,  ControlM    = 13,
   Return      = 13,   ControlN     = 14,  ControlO    = 15,  ControlP    = 16,
   ControlQ    = 17,   ControlR     = 18,  ControlS    = 19,  ControlT    = 20,
   ControlU    = 21,   ControlV     = 22,  ControlW    = 23,  ControlX    = 24,
   ControlY    = 25,   ControlZ     = 26,  Escape      = 27,  Space       = ' ',
   Exclamation = '!',  Quote        = '"', PoundSign   = '#', DollarSign  = '$',
   Percent     = '%',  Ampersand    = '&', SingleQuote = '\'', LeftParen  = '(',
   RightParen  = ')',  Asterisk     = '*', Plus        = '+', Comma       = ',',
   Minus       = '-',  Period       = '.', Slash       = '/', Zero        = '0',
   One         = '1',  Two          = '2', Three       = '3', Four        = '4',
   Five        = '5',  Six          = '6', Seven       = '7', Eight       = '8',
   Nine        = '9',  Colon        = ':', Semicolon   = ';', LessThan    = '<',
   Equals      = '=',  GreaterThan  = '>', Question    = '?', LeftBracket = '[',
   Backslash   = '\\', RightBracket = ']', Carat       = '^', Underscore  = '_',
   Backquote   = '`',  LeftBrace    = '{', VerticalBar = '|', RightBrace  = '}',
   Tilde       = '~',  Delete       = 127, CSI         = 155;

// Latin-1 character constants.
const unsigned char          NBSpace           = 160, InvertedExclamation = 161,
   CentSign           = 162, PoundSterling     = 163, GeneralCurrencySign = 164,
   YenSign            = 165, BrokenVerticalBar = 166, SectionSign         = 167,
   Umlaut             = 168, Copyright         = 169, FeminineOrdinal     = 170,
   LeftAngleQuote     = 171, NotSign           = 172, SoftHyphen          = 173,
   RegisteredTrademark = 174, MacronAccent     = 175, DegreeSign          = 176,
   PlusMinus          = 177, SuperscriptTwo    = 178, SuperscriptThree    = 179,
   AcuteAccent        = 180, MicroSign         = 181, ParagraphSign       = 182,
   MiddleDot          = 183, Cedilla           = 184, SuperscriptOne      = 185,
   MasculineOrdinal   = 186, RightAngleQuote   = 187, OneFourth           = 188,
   OneHalf            = 189, ThreeFourths      = 190, InvertedQuestion    = 191,
   A_grave            = 192, A_acute           = 193, A_circumflex        = 194,
   A_tilde            = 195, A_umlaut          = 196, A_ring              = 197,
   AE_ligature        = 198, C_cedilla         = 199, E_grave             = 200,
   E_acute            = 201, E_circumflex      = 202, E_umlaut            = 203,
   I_grave            = 204, I_acute           = 205, I_circumflex        = 206,
   I_umlaut           = 207, ETH_Icelandic     = 208, N_tilde             = 209,
   O_grave            = 210, O_acute           = 211, O_circumflex        = 212,
   O_tilde            = 213, O_umlaut          = 214, MultiplySign        = 215,
   O_slash            = 216, U_grave           = 217, U_acute             = 218,
   U_circumflex       = 219, U_umlaut          = 220, Y_acute             = 221,
   THORN_Icelandic    = 222, sz_ligature       = 223, a_grave             = 224,
   a_acute            = 225, a_circumflex      = 226, a_tilde             = 227,
   a_umlaut           = 228, a_ring            = 229, ae_ligature         = 230,
   c_cedilla          = 231, e_grave           = 232, e_acute             = 233,
   e_circumflex       = 234, e_umlaut          = 235, i_grave             = 236,
   i_acute            = 237, i_circumflex      = 238, i_umlaut            = 239,
   eth_Icelandic      = 240, n_tilde           = 241, o_grave             = 242,
   o_acute            = 243, o_circumflex      = 244, o_tilde             = 245,
   o_umlaut           = 246, DivisionSign      = 247, o_slash             = 248,
   u_grave            = 249, u_acute           = 250, u_circumflex        = 251,
   u_umlaut           = 252, y_acute           = 253, thorn_Icelandic     = 254,
   y_umlaut           = 255;

#endif // constants.h
