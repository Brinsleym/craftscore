# Security overview

ScoreCraft's attack surface is untrusted documents (PDF, images, audio, `.4sc`, setlist bundles),
untrusted network responses (IMSLP, catalogues, sync, eSCL), untrusted device input (MIDI, HID),
and automation. The rules: never crash on input, cap every resource sized by input, confine the
filesystem, keep the network opt-in, and never lose a user's markings.

Policy and reporting: [`SECURITY.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/SECURITY.md). Pages in this section describe the planned
controls; as code lands, each control gets a "status" line naming the test that proves it.
