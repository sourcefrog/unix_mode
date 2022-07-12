// Copyright 2022 Martin Pool
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests that do not depend on the target OS's behavior, and that can run on any OS.

use unix_mode::Type;

#[test]
fn permissions_to_type() {
    let cases = [(0o0010000, Type::Fifo), (0o0100666, Type::File)];
    for (mode, expected_type) in cases {
        let t = Type::from(mode);
        assert_eq!(t, expected_type);
        assert_eq!(unix_mode::is_fifo(mode), t == Type::Fifo);
        assert_eq!(unix_mode::is_file(mode), t == Type::File);
    }
}
