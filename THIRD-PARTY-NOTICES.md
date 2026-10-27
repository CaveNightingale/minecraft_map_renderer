# Third-party notices

This repository's own code is dedicated to the public domain under
[CC0 1.0 Universal](LICENSE). That dedication covers **only** the code written
for this project. It does not relicense anything listed below: each third-party
component keeps its own license, and the attribution requirements of those
licenses still apply to every copy you distribute.

The sections below separate components that end up inside a published artifact
from those used only while building or testing.

## 1. Rust CLI (linked into the published binary)

`cargo build --release` statically links the crates below into
`minecraft-map-renderer`. Every one of them is permissive. You may choose either
license wherever `OR` is given; the copyright notices and the full texts are in
the appendices.

| Crate | Version | License |
| --- | --- | --- |
| `anstream` | 1.0.0 | MIT OR Apache-2.0 |
| `anstyle` | 1.0.14 | MIT OR Apache-2.0 |
| `anstyle-parse` | 1.0.0 | MIT OR Apache-2.0 |
| `anstyle-query` | 1.1.5 | MIT OR Apache-2.0 |
| `anyhow` | 1.0.104 | MIT OR Apache-2.0 |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 |
| `cc` | 1.10.0 | MIT OR Apache-2.0 |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 |
| `clap` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_builder` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_derive` | 4.6.7 | MIT OR Apache-2.0 |
| `clap_lex` | 1.1.1 | MIT OR Apache-2.0 |
| `colorchoice` | 1.0.5 | MIT OR Apache-2.0 |
| `console` | 0.16.6 | MIT |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 |
| `digest` | 0.10.7 | MIT OR Apache-2.0 |
| `find-msvc-tools` | 0.1.15 | MIT OR Apache-2.0 |
| `generic-array` | 0.14.7 | MIT |
| `heck` | 0.5.0 | MIT OR Apache-2.0 |
| `indicatif` | 0.18.6 | MIT |
| `is_terminal_polyfill` | 1.70.2 | MIT OR Apache-2.0 |
| `itoa` | 1.0.18 | MIT OR Apache-2.0 |
| `libc` | 0.2.191 | MIT OR Apache-2.0 |
| `libdeflate-sys` | 1.26.1 | Apache-2.0 (bundles MIT libdeflate) |
| `libdeflater` | 1.26.1 | Apache-2.0 |
| `lz4-java-wrc` | 0.2.0 | MIT |
| `lz4_flex` | 0.11.6 | MIT |
| `memchr` | 2.8.3 | Unlicense OR MIT |
| `portable-atomic` | 1.15.0 | Apache-2.0 OR MIT |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 |
| `quote` | 1.0.47 | MIT OR Apache-2.0 |
| `rapidhash` | 4.5.1 | MIT OR Apache-2.0 |
| `rustversion` | 1.0.23 | MIT OR Apache-2.0 |
| `serde` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 |
| `shlex` | 2.0.1 | MIT OR Apache-2.0 |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 |
| `strsim` | 0.11.1 | MIT |
| `syn` | 3.0.9 | MIT OR Apache-2.0 |
| `tinyvec` | 1.13.3 | Zlib OR Apache-2.0 OR MIT |
| `twox-hash` | 1.6.3 | MIT |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 |
| `unicode-width` | 0.2.2 | MIT OR Apache-2.0 |
| `unit-prefix` | 0.5.2 | MIT |
| `utf8parse` | 0.2.2 | Apache-2.0 OR MIT |
| `version_check` | 0.9.5 | MIT OR Apache-2.0 |
| `zmij` | 1.0.23 | MIT |

### Copyright notices

These crates ship an explicit copyright line in their own license file; it is
reproduced here as their licenses require. Crates whose upstream `LICENSE` file
carries no separate copyright line (several of the `dtolnay` and `rust-cli`
crates) are covered by the license texts in the appendices alone, exactly as
their authors published them. To reproduce every upstream file verbatim, run
`cargo vendor`.

| Crate | Copyright |
| --- | --- |
| `anstream`, `anstyle`, `anstyle-parse`, `anstyle-query`, `clap`, `clap_builder`, `clap_derive`, `clap_lex`, `colorchoice`, `is_terminal_polyfill` | Copyright (c) Individual contributors |
| `block-buffer` | Copyright (c) 2018-2019 The RustCrypto Project Developers |
| `cc`, `cfg-if`, `find-msvc-tools` | Copyright (c) 2014 Alex Crichton |
| `console`, `indicatif` | Copyright (c) 2017 Armin Ronacher |
| `cpufeatures` | Copyright (c) 2020-2025 The RustCrypto Project Developers |
| `crypto-common` | Copyright (c) 2021 RustCrypto Developers |
| `digest` | Copyright (c) 2017 Artyom Pavlov |
| `generic-array` | Copyright (c) 2015 Bartłomiej Kamiński |
| `heck` | Copyright (c) 2015 The Rust Project Developers |
| `libc`, `unicode-width` | Copyright (c) The Rust Project Developers |
| `lz4-java-wrc` | Copyright (c) 2021 Alexandre Blazart |
| `lz4_flex` | Copyright (c) 2020 Pascal Seitz |
| `memchr` | Copyright (c) 2015 Andrew Gallant |
| `rapidhash` | Copyright (c) 2024 Liam Gray |
| `sha2` | Copyright (c) 2006-2009 Graydon Hoare |
| `shlex` | Copyright 2015 Nicholas Allegra (comex) |
| `static_assertions` | Copyright (c) 2017 Nikolai Vazquez |
| `strsim` | Copyright (c) 2015 Danny Guo |
| `twox-hash` | Copyright (c) 2015 Jake Goulding |
| `typenum` | Copyright (c) 2014 Paho Lurie-Gregg |
| `unicode-ident` | Copyright © 1991-2023 Unicode, Inc. (Unicode License V3) |
| `unit-prefix` | Copyright (c) 2024 Benjamin Sago, Fabio Valentini |
| `utf8parse` | Copyright (c) 2016 Joe Wilm |
| `version_check` | Copyright (c) 2017-2018 Sergio Benitez |

`libdeflate-sys` compiles the C library **libdeflate** (MIT, Copyright 2016 Eric
Biggers and 2024 Google LLC) and links it statically. Its notice is reproduced in
[Appendix E](#appendix-e--libdeflate).

## 2. Rust dependencies used only for development

Not linked into any published artifact, listed for completeness:

| Crate | License |
| --- | --- |
| `flate2` | MIT OR Apache-2.0 |
| `miniz_oxide` | MIT OR Zlib OR Apache-2.0 |
| `zlib-rs` | Zlib |
| `tempfile`, `getrandom` | MIT OR Apache-2.0 |
| `r-efi` | MIT OR Apache-2.0 OR LGPL-2.1-or-later |

`r-efi` is dual/triple licensed and is only reached on UEFI targets; the MIT
option applies. It is not distributed.

## 3. Fabric exporter (Java)

| Component | License | Distributed by this project |
| --- | --- | --- |
| Fabric Loader 0.19.5 | Apache-2.0 | No, the user installs it |
| Fabric API 0.145.1+26.1 | Apache-2.0 | No, the user installs it |
| Fabric Loom 1.17.21 | MIT | No, build tooling |
| Gradle & the Gradle wrapper | Apache-2.0 | Yes: `gradlew`, `gradlew.bat`, `gradle-wrapper.jar` |
| JUnit Jupiter / Platform 6.0.1 | Eclipse Public License 2.0 | No, test scope only |
| Minecraft 26.1 | Proprietary (Mojang) | No, never redistributed |

The built mod jar contains only this project's own classes and resources. It does
not bundle Minecraft, Fabric Loader, Fabric API or Mixin code, so those licenses
impose no obligations on the jar itself.

EPL-2.0 is a weak copyleft license, but it only applies to JUnit's own source
files. Depending on JUnit from test code does not affect this project's license.

## 4. npm package (`web/`)

The published package contains only the compiled output of `web/src` and has **no
runtime dependencies**.

| Package | License | Scope |
| --- | --- | --- |
| TypeScript 5.9.3 | Apache-2.0 | devDependency |
| `@playwright/test`, `playwright`, `playwright-core` 1.64.1 | Apache-2.0 | devDependency |

## 5. Minecraft

This project is not affiliated with, endorsed by or sponsored by Mojang Studios
or Microsoft. *Minecraft* is a trademark of Mojang Synergies AB. Minecraft's code
and assets are not distributed here and are not covered by the CC0 dedication.
You must own a legitimate copy of the game to use the exporter.

## Appendix A. MIT License

```
Permission is hereby granted, free of charge, to any
person obtaining a copy of this software and associated
documentation files (the "Software"), to deal in the
Software without restriction, including without
limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software
is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice
shall be included in all copies or substantial portions
of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
DEALINGS IN THE SOFTWARE.
```

## Appendix B. Apache License 2.0

```
                              Apache License
                        Version 2.0, January 2004
                     http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.

   "License" shall mean the terms and conditions for use, reproduction,
   and distribution as defined by Sections 1 through 9 of this document.

   "Licensor" shall mean the copyright owner or entity authorized by
   the copyright owner that is granting the License.

   "Legal Entity" shall mean the union of the acting entity and all
   other entities that control, are controlled by, or are under common
   control with that entity. For the purposes of this definition,
   "control" means (i) the power, direct or indirect, to cause the
   direction or management of such entity, whether by contract or
   otherwise, or (ii) ownership of fifty percent (50%) or more of the
   outstanding shares, or (iii) beneficial ownership of such entity.

   "You" (or "Your") shall mean an individual or Legal Entity
   exercising permissions granted by this License.

   "Source" form shall mean the preferred form for making modifications,
   including but not limited to software source code, documentation
   source, and configuration files.

   "Object" form shall mean any form resulting from mechanical
   transformation or translation of a Source form, including but
   not limited to compiled object code, generated documentation,
   and conversions to other media types.

   "Work" shall mean the work of authorship, whether in Source or
   Object form, made available under the License, as indicated by a
   copyright notice that is included in or attached to the work
   (an example is provided in the Appendix below).

   "Derivative Works" shall mean any work, whether in Source or Object
   form, that is based on (or derived from) the Work and for which the
   editorial revisions, annotations, elaborations, or other modifications
   represent, as a whole, an original work of authorship. For the purposes
   of this License, Derivative Works shall not include works that remain
   separable from, or merely link (or bind by name) to the interfaces of,
   the Work and Derivative Works thereof.

   "Contribution" shall mean any work of authorship, including
   the original version of the Work and any modifications or additions
   to that Work or Derivative Works thereof, that is intentionally
   submitted to Licensor for inclusion in the Work by the copyright owner
   or by an individual or Legal Entity authorized to submit on behalf of
   the copyright owner. For the purposes of this definition, "submitted"
   means any form of electronic, verbal, or written communication sent
   to the Licensor or its representatives, including but not limited to
   communication on electronic mailing lists, source code control systems,
   and issue tracking systems that are managed by, or on behalf of, the
   Licensor for the purpose of discussing and improving the Work, but
   excluding communication that is conspicuously marked or otherwise
   designated in writing by the copyright owner as "Not a Contribution."

   "Contributor" shall mean Licensor and any individual or Legal Entity
   on behalf of whom a Contribution has been received by Licensor and
   subsequently incorporated within the Work.

2. Grant of Copyright License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   copyright license to reproduce, prepare Derivative Works of,
   publicly display, publicly perform, sublicense, and distribute the
   Work and such Derivative Works in Source or Object form.

3. Grant of Patent License. Subject to the terms and conditions of
   this License, each Contributor hereby grants to You a perpetual,
   worldwide, non-exclusive, no-charge, royalty-free, irrevocable
   (except as stated in this section) patent license to make, have made,
   use, offer to sell, sell, import, and otherwise transfer the Work,
   where such license applies only to those patent claims licensable
   by such Contributor that are necessarily infringed by their
   Contribution(s) alone or by combination of their Contribution(s)
   with the Work to which such Contribution(s) was submitted. If You
   institute patent litigation against any entity (including a
   cross-claim or counterclaim in a lawsuit) alleging that the Work
   or a Contribution incorporated within the Work constitutes direct
   or contributory patent infringement, then any patent licenses
   granted to You under this License for that Work shall terminate
   as of the date such litigation is filed.

4. Redistribution. You may reproduce and distribute copies of the
   Work or Derivative Works thereof in any medium, with or without
   modifications, and in Source or Object form, provided that You
   meet the following conditions:

   (a) You must give any other recipients of the Work or
       Derivative Works a copy of this License; and

   (b) You must cause any modified files to carry prominent notices
       stating that You changed the files; and

   (c) You must retain, in the Source form of any Derivative Works
       that You distribute, all copyright, patent, trademark, and
       attribution notices from the Source form of the Work,
       excluding those notices that do not pertain to any part of
       the Derivative Works; and

   (d) If the Work includes a "NOTICE" text file as part of its
       distribution, then any Derivative Works that You distribute must
       include a readable copy of the attribution notices contained
       within such NOTICE file, excluding those notices that do not
       pertain to any part of the Derivative Works, in at least one
       of the following places: within a NOTICE text file distributed
       as part of the Derivative Works; within the Source form or
       documentation, if provided along with the Derivative Works; or,
       within a display generated by the Derivative Works, if and
       wherever such third-party notices normally appear. The contents
       of the NOTICE file are for informational purposes only and
       do not modify the License. You may add Your own attribution
       notices within Derivative Works that You distribute, alongside
       or as an addendum to the NOTICE text from the Work, provided
       that such additional attribution notices cannot be construed
       as modifying the License.

   You may add Your own copyright statement to Your modifications and
   may provide additional or different license terms and conditions
   for use, reproduction, or distribution of Your modifications, or
   for any such Derivative Works as a whole, provided Your use,
   reproduction, and distribution of the Work otherwise complies with
   the conditions stated in this License.

5. Submission of Contributions. Unless You explicitly state otherwise,
   any Contribution intentionally submitted for inclusion in the Work
   by You to the Licensor shall be under the terms and conditions of
   this License, without any additional terms or conditions.
   Notwithstanding the above, nothing herein shall supersede or modify
   the terms of any separate license agreement you may have executed
   with Licensor regarding such Contributions.

6. Trademarks. This License does not grant permission to use the trade
   names, trademarks, service marks, or product names of the Licensor,
   except as required for reasonable and customary use in describing the
   origin of the Work and reproducing the content of the NOTICE file.

7. Disclaimer of Warranty. Unless required by applicable law or
   agreed to in writing, Licensor provides the Work (and each
   Contributor provides its Contributions) on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
   implied, including, without limitation, any warranties or conditions
   of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
   PARTICULAR PURPOSE. You are solely responsible for determining the
   appropriateness of using or redistributing the Work and assume any
   risks associated with Your exercise of permissions under this License.

8. Limitation of Liability. In no event and under no legal theory,
   whether in tort (including negligence), contract, or otherwise,
   unless required by applicable law (such as deliberate and grossly
   negligent acts) or agreed to in writing, shall any Contributor be
   liable to You for damages, including any direct, indirect, special,
   incidental, or consequential damages of any character arising as a
   result of this License or out of the use or inability to use the
   Work (including but not limited to damages for loss of goodwill,
   work stoppage, computer failure or malfunction, or any and all
   other commercial damages or losses), even if such Contributor
   has been advised of the possibility of such damages.

9. Accepting Warranty or Additional Liability. While redistributing
   the Work or Derivative Works thereof, You may choose to offer,
   and charge a fee for, acceptance of support, warranty, indemnity,
   or other liability obligations and/or rights consistent with this
   License. However, in accepting such obligations, You may act only
   on Your own behalf and on Your sole responsibility, not on behalf
   of any other Contributor, and only if You agree to indemnify,
   defend, and hold each Contributor harmless for any liability
   incurred by, or claims asserted against, such Contributor by reason
   of your accepting any such warranty or additional liability.

END OF TERMS AND CONDITIONS
```

## Appendix C. zlib License

```
Copyright (c) 2019 Daniel "Lokathor" Gee.

This software is provided 'as-is', without any express or implied warranty. In no event will the authors be held liable for any damages arising from the use of this software.

Permission is granted to anyone to use this software for any purpose, including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:

1. The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation would be appreciated but is not required.

2. Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.

3. This notice may not be removed or altered from any source distribution.
```

## Appendix D. Unicode License V3

```
UNICODE LICENSE V3

COPYRIGHT AND PERMISSION NOTICE

Copyright © 1991-2023 Unicode, Inc.

NOTICE TO USER: Carefully read the following legal agreement. BY
DOWNLOADING, INSTALLING, COPYING OR OTHERWISE USING DATA FILES, AND/OR
SOFTWARE, YOU UNEQUIVOCALLY ACCEPT, AND AGREE TO BE BOUND BY, ALL OF THE
TERMS AND CONDITIONS OF THIS AGREEMENT. IF YOU DO NOT AGREE, DO NOT
DOWNLOAD, INSTALL, COPY, DISTRIBUTE OR USE THE DATA FILES OR SOFTWARE.

Permission is hereby granted, free of charge, to any person obtaining a
copy of data files and any associated documentation (the "Data Files") or
software and any associated documentation (the "Software") to deal in the
Data Files or Software without restriction, including without limitation
the rights to use, copy, modify, merge, publish, distribute, and/or sell
copies of the Data Files or Software, and to permit persons to whom the
Data Files or Software are furnished to do so, provided that either (a)
this copyright and permission notice appear with all copies of the Data
Files or Software, or (b) this copyright and permission notice appear in
associated Documentation.

THE DATA FILES AND SOFTWARE ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY
KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT OF
THIRD PARTY RIGHTS.

IN NO EVENT SHALL THE COPYRIGHT HOLDER OR HOLDERS INCLUDED IN THIS NOTICE
BE LIABLE FOR ANY CLAIM, OR ANY SPECIAL INDIRECT OR CONSEQUENTIAL DAMAGES,
OR ANY DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS,
WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION,
ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THE DATA
FILES OR SOFTWARE.

Except as contained in this notice, the name of a copyright holder shall
not be used in advertising or otherwise to promote the sale, use or other
dealings in these Data Files or Software without prior written
authorization of the copyright holder.
```

## Appendix E. libdeflate

```
Copyright 2016 Eric Biggers
Copyright 2024 Google LLC

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation files
(the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge,
publish, distribute, sublicense, and/or sell copies of the Software,
and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
