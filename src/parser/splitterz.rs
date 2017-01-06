use std::borrow::Cow;
use std::io::{self, Read, BufRead, BufReader};
use std::fs::File;
use std::result::Result as StdResult;
use std::num::ParseIntError;
use {Run, time_span, Image, TimeSpan, Time, Segment};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Empty
        ExpectedCategoryName
        ExpectedAttemptCount
        ExpectedSplitName
        ExpectedSplitTime
        ExpectedBestSegment
        Attempt(err: ParseIntError) {
            from()
        }
        Time(err: time_span::ParseError) {
            from()
        }
        Io(err: io::Error) {
            from()
        }
    }
}

pub type Result<T> = StdResult<T, Error>;

fn unescape(text: &str) -> Cow<str> {
    if text.contains('‡') {
        text.replace('‡', ",").into()
    } else {
        text.into()
    }
}

pub fn parse<R: BufRead>(source: R, load_icons: bool) -> Result<Run> {
    let mut run = Run::new(Vec::new());

    let mut icon_buf = Vec::new();

    let mut lines = source.lines();
    let line = lines.next().ok_or(Error::Empty)??;
    let line = line.trim();
    let mut splits = line.split(',');
    // Title Stuff here, do later
    run.set_category_name(unescape(splits.next().ok_or(Error::ExpectedCategoryName)?));
    run.set_attempt_count(splits.next().ok_or(Error::ExpectedAttemptCount)?.parse()?);

    for line in lines {
        let line = line?;
        let line = line.trim();
        if !line.is_empty() {
            let mut splits = line.split(',');

            let mut segment = Segment::new(unescape(splits.next()
                .ok_or(Error::ExpectedSplitName)?));

            let time: TimeSpan = splits.next().ok_or(Error::ExpectedSplitTime)?.parse()?;
            if time != TimeSpan::zero() {
                segment.set_personal_best_split_time(Time::new().with_real_time(Some(time)));
            }

            let time: TimeSpan = splits.next().ok_or(Error::ExpectedBestSegment)?.parse()?;
            if time != TimeSpan::zero() {
                segment.set_best_segment_time(Time::new().with_real_time(Some(time)));
            }

            if load_icons {
                if let Some(icon_path) = splits.next() {
                    if !icon_path.is_empty() {
                        if let Ok(file) = File::open(unescape(icon_path).as_ref()) {
                            icon_buf.clear();
                            if BufReader::new(file).read_to_end(&mut icon_buf).is_ok() {
                                segment.set_icon(Image::new(&icon_buf));
                            }
                        }
                    }
                }
            }

            run.push_segment(segment);
        }
    }

    Ok(run)
}
