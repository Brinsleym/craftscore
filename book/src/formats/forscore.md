# forScore `.4sc` import

enScore migrates forScore libraries through `.4sc` files, carrying pages, ink, stamps and layers.
There is no public specification, so ScoreCraft's reader (`scorecraft-fsc`) follows the
two-team black-box rule: an analyst who owns forScore makes sample files, compares them and writes
a description in `docs/spec/formats/`; a different implementer writes the reader from that
description and the samples only. The reader is read-only and reports anything it does not
understand. Rules: [`docs/clean-room.md` §5](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/clean-room.md#5-interchange-formats-without-a-public-spec).
