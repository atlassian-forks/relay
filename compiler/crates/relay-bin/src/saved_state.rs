/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use std::fs;
use std::io;
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use relay_compiler::compiler_state::CompilerState;

use crate::errors::Error;

static NEXT_TEMP: AtomicUsize = AtomicUsize::new(0);

// Keep the temporary file on the destination filesystem so replacement is atomic.
// A private, exclusively created directory avoids clobbering another invocation's
// temporary output. This is local checkpoint replacement, not cache publication.
pub(crate) fn save(state: &CompilerState, path: &Path) -> Result<(), Error> {
    let write_error = |err: io::Error| Error::CompilerError {
        details: format!("Unable to save checkpoint {}: {err}", path.display()),
    };
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let directory = loop {
        let directory = parent.join(format!(
            ".relay-state-{}-{}",
            std::process::id(),
            NEXT_TEMP.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&directory) {
            Ok(()) => break directory,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(write_error(err)),
        }
    };
    let temporary = directory.join("state.bin");
    let result = state
        .serialize_to_file(&temporary)
        .map_err(|err| Error::CompilerError {
            details: format!("{err}"),
        })
        .and_then(|()| fs::rename(&temporary, path).map_err(write_error));
    // Cleanup must not turn a successfully replaced checkpoint into a failed run.
    let _ = fs::remove_file(&temporary);
    let _ = fs::remove_dir(&directory);
    result
}
