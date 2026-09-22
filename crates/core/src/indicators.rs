use serde_json::json;
use std::collections::BTreeMap;

use crate::engines::*;
use crate::math::*;
use crate::model::*;

// ---------------------------------------------------------------------------
// Builder helpers for catalog definitions
// ---------------------------------------------------------------------------

fn num_in(key: &str, label: &str, def: f64, min: f64, max: f64, step: f64) -> InputDef {
    InputDef {
        key: key.into(),
        label: label.into(),
        kind: "number".into(),
        def: json!(def),
        min: Some(min),
        max: Some(max),
        step: Some(step),
        options: vec![],
    }
}

fn source_in(def: &str) -> InputDef {
    let opts: Vec<InputOption> = [
        ("close", "Close"),
        ("open", "Open"),
        ("high", "High"),
        ("low", "Low"),
        ("hl2", "HL2"),
        ("hlc3", "HLC3"),
        ("hlcc4", "HLCC4"),
    ]
    .iter()
    .map(|(v, l)| InputOption { value: (*v).into(), label: (*l).into() })
    .collect();
    InputDef {
        key: "source".into(),
        label: "Source".into(),
        kind: "source".into(),
        def: json!(def),
        min: None,
        max: None,
        step: None,
        options: opts,
    }
}

fn check_in(key: &str, label: &str, def: bool) -> InputDef {
    InputDef {
        key: key.into(),
        label: label.into(),
        kind: "bool".into(),
        def: json!(def),
        min: None,
        max: None,
        step: None,
        options: vec![],
    }
}

fn enum_in(key: &str, label: &str, def: &str, opts: &[(&str, &str)]) -> InputDef {
    InputDef {
        key: key.into(),
        label: label.into(),
        kind: "enum".into(),
        def: json!(def),
        min: None,
        max: None,
        step: None,
        options: opts
            .iter()
            .map(|(v, l)| InputOption { value: (*v).into(), label: (*l).into() })
            .collect(),
    }
}

fn color_st(key: &str, label: &str, def: &str) -> StyleDef {
    StyleDef {
        key: key.into(),
        label: label.into(),
        kind: "color".into(),
        def: json!(def),
        min: None,
        max: None,
        step: None,
    }
}

fn num_st(key: &str, label: &str, def: f64, min: f64, max: f64, step: f64) -> StyleDef {
    StyleDef {
        key: key.into(),
        label: label.into(),
        kind: "number".into(),
        def: json!(def),
        min: Some(min),
        max: Some(max),
        step: Some(step),
    }
}

fn bool_st(key: &str, label: &str, def: bool) -> StyleDef {
    StyleDef {
        key: key.into(),
        label: label.into(),
        kind: "bool".into(),
        def: json!(def),
        min: None,
        max: None,
        step: None,
    }
}

fn def(
    id: &str,
    name: &str,
    full: &str,
    cat: &str,
    kind: IndType,
    format: Option<&str>,
    inputs: Vec<InputDef>,
    style: Vec<StyleDef>,
) -> IndicatorDef {
    IndicatorDef {
        id: id.into(),
        name: name.into(),
        full_name: full.into(),
        cat: cat.into(),
        kind,
        format: format.map(|s| s.into()),
        hidden: false,
        inputs,
        style,
    }
}

fn def_hidden(
    id: &str,
    name: &str,
    full: &str,
    cat: &str,
    kind: IndType,
    format: Option<&str>,
    inputs: Vec<InputDef>,
    style: Vec<StyleDef>,
) -> IndicatorDef {
    let mut d = def(id, name, full, cat, kind, format, inputs, style);
    d.hidden = true;
    d
}

macro_rules! e {
    ($def:expr, $compute:expr, $reading:expr) => {
        IndicatorEntry { def: $def, compute: $compute, markers: None, reading: $reading }
    };
    ($def:expr, $compute:expr, $markers:expr, $reading:expr) => {
        IndicatorEntry { def: $def, compute: $compute, markers: Some($markers), reading: $reading }
    };
}

// ---------------------------------------------------------------------------
// Standard indicator computes
// ---------------------------------------------------------------------------

fn compute_ema(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#2962ff");
    let lw = num(o, "lineWidth", 1.0);
    let src = src_arr(c, &strv(o, "source", "close"));
    let arr = ema_arr(&src, int(o, "length", 9));
    vec![build_series(c, &arr, &color, SeriesKind::Line, lw)]
}

fn compute_ma(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#ff6d00");
    let lw = num(o, "lineWidth", 1.0);
    let src = src_arr(c, &strv(o, "source", "close"));
    let arr = sma_arr(&src, int(o, "length", 20));
    vec![build_series(c, &arr, &color, SeriesKind::Line, lw)]
}

fn compute_smma(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#ffca28");
    let lw = num(o, "lineWidth", 1.0);
    let src = src_arr(c, &strv(o, "source", "close"));
    let arr = wilder_arr(&src, int(o, "length", 20));
    vec![build_series(c, &arr, &color, SeriesKind::Line, lw)]
}

fn compute_hma(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#00bcd4");
    let lw = num(o, "lineWidth", 1.0);
    let len = int(o, "length", 20).max(1);
    let src = src_arr(c, &strv(o, "source", "close"));
    let half = (len / 2).max(1);
    let sq = (len as f64).sqrt().round() as i64;
    let wma = |vals: &[f64], p: i64| -> Vec<Option<f64>> {
        let n = vals.len();
        let mut out = vec![None; n];
        if p <= 0 {
            return out;
        }
        let p = p as usize;
        let denom = (p * (p + 1) / 2) as f64;
        for i in (p - 1)..n {
            let mut s = 0.0;
            for j in 0..p {
                s += vals[i - j] * (p - j) as f64;
            }
            out[i] = Some(s / denom);
        }
        out
    };
    let w1 = wma(&src, half);
    let w2 = wma(&src, len);
    let raw: Vec<Option<f64>> = (0..src.len())
        .map(|i| match (w1[i], w2[i]) {
            (Some(a), Some(b)) => Some(2.0 * a - b),
            _ => None,
        })
        .collect();
    let mut out = vec![None; src.len()];
    let valid: Vec<f64> = raw.iter().map(|v| v.unwrap_or(0.0)).collect();
    let sm = wma(&valid, sq);
    for i in 0..src.len() {
        if raw[i].is_some() {
            out[i] = sm[i];
        }
    }
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_ao(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let med: Vec<f64> = c.iter().map(|x| (x.high + x.low) / 2.0).collect();
    let f = sma_arr(&med, int(o, "fast", 5));
    let s = sma_arr(&med, int(o, "slow", 34));
    let up = strv(o, "upColor", "#26a69a");
    let down = strv(o, "downColor", "#ef5350");
    let mut data = Vec::new();
    let mut prev: Option<f64> = None;
    for i in 0..c.len() {
        if let (Some(a), Some(b)) = (f[i], s[i]) {
            let v = a - b;
            let is_up = prev.map(|p| v >= p).unwrap_or(true);
            data.push(Point { time: c[i].time, value: v, color: Some(if is_up { up.clone() } else { down.clone() }) });
            prev = Some(v);
        }
    }
    let mut s = SeriesOut::hist(&up);
    s.data = data;
    vec![s]
}

fn compute_atr(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#7e57c2");
    let lw = num(o, "lineWidth", 1.0);
    let arr = wilder_arr(&tr_arr(c), int(o, "length", 14));
    vec![build_series(c, &arr, &color, SeriesKind::Line, lw)]
}

fn compute_adx(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let len = int(o, "length", 14).max(1);
    let tr = tr_arr(c);
    let mut up = vec![0.0; n];
    let mut dn = vec![0.0; n];
    for i in 1..n {
        let um = c[i].high - c[i - 1].high;
        let dm = c[i - 1].low - c[i].low;
        up[i] = if um > dm && um > 0.0 { um } else { 0.0 };
        dn[i] = if dm > um && dm > 0.0 { dm } else { 0.0 };
    }
    let sutr = wilder_arr(&tr, len);
    let sup = wilder_arr(&up, len);
    let sdn = wilder_arr(&dn, len);
    let mut di_p = vec![None; n];
    let mut di_m = vec![None; n];
    let mut dx = vec![None; n];
    for i in 0..n {
        if let (Some(t), Some(p), Some(m)) = (sutr[i], sup[i], sdn[i]) {
            if t != 0.0 {
                let dp = 100.0 * p / t;
                let dm = 100.0 * m / t;
                di_p[i] = Some(dp);
                di_m[i] = Some(dm);
                let sum = dp + dm;
                dx[i] = Some(if sum != 0.0 { 100.0 * (dp - dm).abs() / sum } else { 0.0 });
            }
        }
    }
    let dx_filled: Vec<f64> = dx.iter().map(|v| v.unwrap_or(0.0)).collect();
    let adx_arr = wilder_arr(&dx_filled, len);
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &adx_arr, &strv(o, "adxColor", "#e040fb"), SeriesKind::Line, lw),
        build_series(c, &di_p, &strv(o, "diPlusColor", "#26a69a"), SeriesKind::Line, lw),
        build_series(c, &di_m, &strv(o, "diMinusColor", "#ef5350"), SeriesKind::Line, lw),
    ]
}

fn compute_bollinger_b(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let mult = num(o, "mult", 2.0);
    let cl = src_arr(c, "close");
    let mid = sma_arr(&cl, len);
    let sd = stdev_arr(&cl, len);
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    let mut data = Vec::new();
    for i in 0..c.len() {
        if let (Some(m), Some(s)) = (mid[i], sd[i]) {
            let up = m + mult * s;
            let lo = m - mult * s;
            let range = up - lo;
            data.push(Point { time: c[i].time, value: if range != 0.0 { (cl[i] - lo) / range } else { 0.0 }, color: None });
        }
    }
    let mut s = SeriesOut::line(&color, lw);
    s.data = data;
    vec![s]
}

fn compute_bbpct(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    // Matches the old app: rolling mean/stdev, %B, then SMA smoothing of the valid tail.
    let len = int(o, "length", 20).max(1) as usize;
    let mult = if num(o, "mult", 2.0) > 0.0 { num(o, "mult", 2.0) } else { 2.0 };
    let smooth = int(o, "smooth", 1).max(1) as usize;
    let src = src_arr(c, &strv(o, "source", "close"));
    let n = c.len();
    let mut b = vec![None; n];
    if n >= len {
        let mut sum = 0.0;
        let mut sumsq = 0.0;
        for i in 0..n {
            let v = src[i];
            sum += v;
            sumsq += v * v;
            if i >= len {
                let old = src[i - len];
                sum -= old;
                sumsq -= old * old;
            }
            if i + 1 >= len {
                let mean = sum / len as f64;
                let var0 = (sumsq / len as f64 - mean * mean).max(0.0);
                let sd = var0.sqrt();
                let up = mean + mult * sd;
                let lo = mean - mult * sd;
                let range = up - lo;
                b[i] = Some(if range > 0.0 { (v - lo) / range } else { 0.5 });
            }
        }
    }
    let start = (len - 1).min(n);
    let valid: Vec<f64> = b[start..].iter().map(|v| v.unwrap_or(f64::NAN)).collect();
    let valid_times: Vec<i64> = c[start..].iter().map(|x| x.time).collect();
    let sm = sma_arr_tolerant(&valid, smooth);
    let color = strv(o, "color", "#ffb300");
    let lw = num(o, "lineWidth", 1.0);
    let mut data = Vec::new();
    for i in 0..valid.len() {
        if let Some(v) = sm[i] {
            data.push(Point { time: valid_times[i], value: v, color: None });
        }
    }
    let mut s = SeriesOut::line(&color, lw);
    s.data = data;
    vec![s]
}

fn compute_macd(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let cl = src_arr(c, "close");
    let f = ema_arr(&cl, int(o, "fast", 12));
    let s = ema_arr(&cl, int(o, "slow", 26));
    let n = c.len();
    let mut macd = vec![None; n];
    for i in 0..n {
        if let (Some(a), Some(b)) = (f[i], s[i]) {
            macd[i] = Some(a - b);
        }
    }
    let macd_filled: Vec<f64> = macd.iter().map(|v| v.unwrap_or(0.0)).collect();
    let signal = ema_arr(&macd_filled, int(o, "signal", 9));
    let hist_up = strv(o, "histUpColor", "#26a69a");
    let hist_dn = strv(o, "histDownColor", "#ef5350");
    let mut hist = Vec::new();
    let mut prev: Option<f64> = None;
    for i in 0..n {
        if let (Some(m), Some(sig)) = (macd[i], signal[i]) {
            let h = m - sig;
            let up = prev.map(|p| h >= p).unwrap_or(true);
            hist.push(Point { time: c[i].time, value: h, color: Some(if up { hist_up.clone() } else { hist_dn.clone() }) });
            prev = Some(h);
        }
    }
    let macd_color = strv(o, "macdColor", "#2962ff");
    let sig_color = strv(o, "signalColor", "#ff6d00");
    let mut hs = SeriesOut::hist(&hist_up);
    hs.data = hist;
    vec![
        build_series(c, &macd, &macd_color, SeriesKind::Line, 1.0),
        build_series(c, &signal, &sig_color, SeriesKind::Line, 1.0),
        hs,
    ]
}

fn compute_supertrend(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let atr = wilder_arr(&tr_arr(c), int(o, "atrPeriod", 10));
    let factor = num(o, "factor", 3.0);
    let up = strv(o, "upColor", "#26a69a");
    let down = strv(o, "downColor", "#ef5350");
    let mut data = Vec::new();
    let mut fu = 0.0f64;
    let mut fl = 0.0f64;
    let mut trend = 1i32;
    let mut started = false;
    for i in 0..c.len() {
        let a = match atr[i] {
            Some(a) => a,
            None => continue,
        };
        let hl2 = (c[i].high + c[i].low) / 2.0;
        let bu = hl2 + factor * a;
        let bl = hl2 - factor * a;
        if !started {
            fu = bu;
            fl = bl;
            started = true;
        } else {
            fu = if bu < fu || c[i - 1].close > fu { bu } else { fu };
            fl = if bl > fl || c[i - 1].close < fl { bl } else { fl };
            if trend == 1 && c[i].close < fl {
                trend = -1;
            } else if trend == -1 && c[i].close > fu {
                trend = 1;
            }
        }
        data.push(Point { time: c[i].time, value: if trend == 1 { fl } else { fu }, color: Some(if trend == 1 { up.clone() } else { down.clone() }) });
    }
    let mut s = SeriesOut::line(&up, num(o, "lineWidth", 1.0));
    s.data = data;
    vec![s]
}

fn compute_obv(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let color = strv(o, "color", "#26a69a");
    let lw = num(o, "lineWidth", 1.0);
    let n = c.len();
    let mut obv = vec![Some(0.0); n];
    for i in 1..n {
        let prev = obv[i - 1].unwrap();
        let v = if c[i].close > c[i - 1].close {
            prev + c[i].volume
        } else if c[i].close < c[i - 1].close {
            prev - c[i].volume
        } else {
            prev
        };
        obv[i] = Some(v);
    }
    vec![build_series(c, &obv, &color, SeriesKind::Line, lw)]
}

fn compute_bb(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let mult = num(o, "mult", 2.0);
    let cl = src_arr(c, "close");
    let mid = sma_arr(&cl, len);
    let sd = stdev_arr(&cl, len);
    let n = c.len();
    let mut up = vec![None; n];
    let mut lo = vec![None; n];
    for i in 0..n {
        if let (Some(m), Some(s)) = (mid[i], sd[i]) {
            up[i] = Some(m + mult * s);
            lo[i] = Some(m - mult * s);
        }
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &up, &color, SeriesKind::Line, lw),
        build_series(c, &mid, &color, SeriesKind::Line, lw),
        build_series(c, &lo, &color, SeriesKind::Line, lw),
    ]
}

fn compute_bbw(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let mult = num(o, "mult", 2.0);
    let cl = src_arr(c, "close");
    let mid = sma_arr(&cl, len);
    let sd = stdev_arr(&cl, len);
    let n = c.len();
    let mut out = vec![None; n];
    for i in 0..n {
        if let (Some(m), Some(s)) = (mid[i], sd[i]) {
            if m != 0.0 {
                out[i] = Some((2.0 * mult * s) / m * 100.0);
            }
        }
    }
    let color = strv(o, "color", "#ffb300");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_volosc(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let fast = int(o, "fast", 5);
    let slow = int(o, "slow", 20);
    let vol: Vec<f64> = c.iter().map(|x| x.volume).collect();
    let f = sma_arr(&vol, fast);
    let s = sma_arr(&vol, slow);
    let n = c.len();
    let mut out = vec![None; n];
    for i in 0..n {
        if let (Some(a), Some(b)) = (f[i], s[i]) {
            if b != 0.0 {
                out[i] = Some((a - b) / b * 100.0);
            }
        }
    }
    let color = strv(o, "color", "#26a69a");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_ad(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let mut ad = vec![Some(0.0); n];
    for i in 0..n {
        let range = c[i].high - c[i].low;
        let mfm = if range != 0.0 {
            ((c[i].close - c[i].low) - (c[i].high - c[i].close)) / range
        } else {
            0.0
        };
        let prev = if i == 0 { 0.0 } else { ad[i - 1].unwrap() };
        ad[i] = Some(prev + mfm * c[i].volume);
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &ad, &color, SeriesKind::Line, lw)]
}

fn compute_mfi(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let p = int(o, "length", 14).max(1) as usize;
    let n = c.len();
    let tp: Vec<f64> = c.iter().map(|x| (x.high + x.low + x.close) / 3.0).collect();
    let mut out = vec![None; n];
    for i in 0..n {
        if i + 1 < p {
            continue;
        }
        let mut pos = 0.0;
        let mut neg = 0.0;
        for j in (i + 1 - p)..=i {
            if j == 0 {
                continue;
            }
            let mf = tp[j] * c[j].volume;
            if tp[j] > tp[j - 1] {
                pos += mf;
            } else if tp[j] < tp[j - 1] {
                neg += mf;
            }
        }
        out[i] = Some(if neg != 0.0 { 100.0 - 100.0 / (1.0 + pos / neg) } else { 100.0 });
    }
    let color = strv(o, "color", "#ff6d00");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_pvt(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let mut out = vec![Some(0.0); n];
    for i in 1..n {
        let prev = out[i - 1].unwrap();
        let change = if c[i - 1].close != 0.0 {
            (c[i].close - c[i - 1].close) / c[i - 1].close
        } else {
            0.0
        };
        out[i] = Some(prev + change * c[i].volume);
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_dpo(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 21);
    let cl = src_arr(c, "close");
    let sm = sma_arr(&cl, len);
    let shift = len / 2 + 1;
    let n = c.len();
    let mut out = vec![None; n];
    for i in 0..n {
        let j = i as i64 - shift;
        if j >= 0 {
            if let Some(s) = sm[j as usize] {
                out[i] = Some(cl[i] - s);
            }
        }
    }
    let color = strv(o, "color", "#7e57c2");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_ppo(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let cl = src_arr(c, "close");
    let f = ema_arr(&cl, int(o, "fast", 12));
    let s = ema_arr(&cl, int(o, "slow", 26));
    let n = c.len();
    let mut ppo = vec![None; n];
    for i in 0..n {
        if let (Some(a), Some(b)) = (f[i], s[i]) {
            if b != 0.0 {
                ppo[i] = Some((a - b) / b * 100.0);
            }
        }
    }
    let ppo_filled: Vec<f64> = ppo.iter().map(|v| v.unwrap_or(0.0)).collect();
    let signal = ema_arr(&ppo_filled, int(o, "signal", 9));
    let n2 = c.len();
    let mut hist = Vec::new();
    let mut prev: Option<f64> = None;
    let hist_up = strv(o, "histUpColor", "#26a69a");
    let hist_dn = strv(o, "histDownColor", "#ef5350");
    for i in 0..n2 {
        if let (Some(p), Some(sig)) = (ppo[i], signal[i]) {
            let h = p - sig;
            let up = prev.map(|x| h >= x).unwrap_or(true);
            hist.push(Point { time: c[i].time, value: h, color: Some(if up { hist_up.clone() } else { hist_dn.clone() }) });
            prev = Some(h);
        }
    }
    let ppo_color = strv(o, "ppoColor", "#2962ff");
    let sig_color = strv(o, "signalColor", "#ff6d00");
    let mut hs = SeriesOut::hist(&hist_up);
    hs.data = hist;
    vec![
        build_series(c, &ppo, &ppo_color, SeriesKind::Line, 1.0),
        build_series(c, &signal, &sig_color, SeriesKind::Line, 1.0),
        hs,
    ]
}

fn compute_williams_r(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 14);
    let n = c.len();
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let hh = highest_arr(&high, len);
    let ll = lowest_arr(&low, len);
    let mut out = vec![None; n];
    for i in 0..n {
        if let (Some(h), Some(l)) = (hh[i], ll[i]) {
            if h != l {
                out[i] = Some((h - c[i].close) / (h - l) * -100.0);
            }
        }
    }
    let color = strv(o, "color", "#e040fb");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_rsi(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 14).max(1) as usize;
    let src = src_arr(c, &strv(o, "source", "close"));
    let n = src.len();
    let mut out = vec![None; n];
    if n > len {
        let mut gain = 0.0;
        let mut loss = 0.0;
        for i in 1..=len {
            let d = src[i] - src[i - 1];
            if d >= 0.0 {
                gain += d;
            } else {
                loss -= d;
            }
        }
        let mut ag = gain / len as f64;
        let mut al = loss / len as f64;
        out[len] = Some(if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) });
        for i in (len + 1)..n {
            let d = src[i] - src[i - 1];
            let g = if d > 0.0 { d } else { 0.0 };
            let l = if d < 0.0 { -d } else { 0.0 };
            ag = (ag * (len as f64 - 1.0) + g) / len as f64;
            al = (al * (len as f64 - 1.0) + l) / len as f64;
            out[i] = Some(if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) });
        }
    }
    let color = strv(o, "color", "#7e57c2");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_uo(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let p1 = int(o, "fast", 7) as usize;
    let p2 = int(o, "mid", 14) as usize;
    let p3 = int(o, "slow", 28) as usize;
    let n = c.len();
    let mut bp = vec![0.0; n];
    let mut tr = vec![0.0; n];
    for i in 0..n {
        if i == 0 {
            continue;
        }
        let true_low = c[i].low.min(c[i - 1].close);
        let true_high = c[i].high.max(c[i - 1].close);
        bp[i] = c[i].close - true_low;
        tr[i] = true_high - true_low;
    }
    let sum = |arr: &[f64], i: usize, p: usize| -> f64 {
        let mut s = 0.0;
        for j in (i + 1 - p)..=i {
            s += arr[j];
        }
        s
    };
    let mut out = vec![None; n];
    for i in 0..n {
        if i + 1 < p3 {
            continue;
        }
        let a1 = sum(&bp, i, p1) / sum(&tr, i, p1).max(f64::EPSILON);
        let a2 = sum(&bp, i, p2) / sum(&tr, i, p2).max(f64::EPSILON);
        let a3 = sum(&bp, i, p3) / sum(&tr, i, p3).max(f64::EPSILON);
        out[i] = Some(100.0 * (4.0 * a1 + 2.0 * a2 + a3) / 7.0);
    }
    let color = strv(o, "color", "#ff6d00");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_vwap(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let mut out = vec![None; n];
    let mut cum_pv = 0.0;
    let mut cum_v = 0.0;
    for i in 0..n {
        let tp = (c[i].high + c[i].low + c[i].close) / 3.0;
        cum_pv += tp * c[i].volume;
        cum_v += c[i].volume;
        out[i] = Some(if cum_v != 0.0 { cum_pv / cum_v } else { tp });
    }
    let color = strv(o, "color", "#ffb300");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_pc(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let hh = highest_arr(&high, len);
    let ll = lowest_arr(&low, len);
    let color = strv(o, "color", "#26a69a");
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &hh, &color, SeriesKind::Line, lw),
        build_series(c, &ll, &color, SeriesKind::Line, lw),
    ]
}

fn compute_cmf(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20).max(1) as usize;
    let n = c.len();
    let mut mfv = vec![0.0; n];
    for i in 0..n {
        let range = c[i].high - c[i].low;
        let mfm = if range != 0.0 {
            ((c[i].close - c[i].low) - (c[i].high - c[i].close)) / range
        } else {
            0.0
        };
        mfv[i] = mfm * c[i].volume;
    }
    let mut out = vec![None; n];
    for i in 0..n {
        if i + 1 < len {
            continue;
        }
        let mut sum_mfv = 0.0;
        let mut sum_v = 0.0;
        for j in (i + 1 - len)..=i {
            sum_mfv += mfv[j];
            sum_v += c[j].volume;
        }
        out[i] = Some(if sum_v != 0.0 { sum_mfv / sum_v } else { 0.0 });
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_cci(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20).max(1) as usize;
    let n = c.len();
    let tp: Vec<f64> = c.iter().map(|x| (x.high + x.low + x.close) / 3.0).collect();
    let ma = sma_arr(&tp, len as i64);
    let mut out = vec![None; n];
    for i in 0..n {
        if let Some(m) = ma[i] {
            let mut dev = 0.0;
            for j in (i + 1 - len)..=i {
                dev += (tp[j] - m).abs();
            }
            dev /= len as f64;
            out[i] = Some(if dev != 0.0 { (tp[i] - m) / (0.015 * dev) } else { 0.0 });
        }
    }
    let color = strv(o, "color", "#e040fb");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_aroon(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 14).max(1) as usize;
    let n = c.len();
    let mut up = vec![None; n];
    let mut down = vec![None; n];
    for i in 0..n {
        if i + 1 < len {
            continue;
        }
        let start = i + 1 - len;
        let mut hi_idx = start;
        let mut lo_idx = start;
        for j in start..=i {
            if c[j].high >= c[hi_idx].high {
                hi_idx = j;
            }
            if c[j].low <= c[lo_idx].low {
                lo_idx = j;
            }
        }
        up[i] = Some((len as f64 - (i - hi_idx) as f64) / len as f64 * 100.0);
        down[i] = Some((len as f64 - (i - lo_idx) as f64) / len as f64 * 100.0);
    }
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &up, &strv(o, "upColor", "#26a69a"), SeriesKind::Line, lw),
        build_series(c, &down, &strv(o, "downColor", "#ef5350"), SeriesKind::Line, lw),
    ]
}

fn compute_vortex(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 14).max(1) as usize;
    let n = c.len();
    let mut vm_plus = vec![0.0; n];
    let mut vm_minus = vec![0.0; n];
    let mut tr = vec![0.0; n];
    for i in 1..n {
        vm_plus[i] = (c[i].high - c[i - 1].low).abs();
        vm_minus[i] = (c[i].low - c[i - 1].high).abs();
        tr[i] = (c[i].high - c[i].low)
            .max((c[i].high - c[i - 1].close).abs())
            .max((c[i].low - c[i - 1].close).abs());
    }
    let mut vi_p = vec![None; n];
    let mut vi_m = vec![None; n];
    for i in 0..n {
        if i + 1 < len {
            continue;
        }
        let mut sp = 0.0;
        let mut sm = 0.0;
        let mut st = 0.0;
        for j in (i + 1 - len)..=i {
            sp += vm_plus[j];
            sm += vm_minus[j];
            st += tr[j];
        }
        if st != 0.0 {
            vi_p[i] = Some(sp / st);
            vi_m[i] = Some(sm / st);
        }
    }
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &vi_p, &strv(o, "plusColor", "#26a69a"), SeriesKind::Line, lw),
        build_series(c, &vi_m, &strv(o, "minusColor", "#ef5350"), SeriesKind::Line, lw),
    ]
}

fn compute_tsi(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let long = int(o, "long", 25);
    let short = int(o, "short", 13);
    let n = c.len();
    let mut mom = vec![None; n];
    let mut abs_mom = vec![None; n];
    for i in 1..n {
        let m = c[i].close - c[i - 1].close;
        mom[i] = Some(m);
        abs_mom[i] = Some(m.abs());
    }
    let mom_filled: Vec<f64> = mom.iter().map(|v| v.unwrap_or(0.0)).collect();
    let abs_filled: Vec<f64> = abs_mom.iter().map(|v| v.unwrap_or(0.0)).collect();
    let e1 = ema_arr(&mom_filled, long);
    let e2 = ema_arr(&abs_filled, long);
    let s1 = ema_arr(&e1.iter().map(|v| v.unwrap_or(0.0)).collect::<Vec<_>>(), short);
    let s2 = ema_arr(&e2.iter().map(|v| v.unwrap_or(0.0)).collect::<Vec<_>>(), short);
    let mut out = vec![None; n];
    for i in 0..n {
        if e1[i].is_some() && e2[i].is_some() {
            if let (Some(a), Some(b)) = (s1[i], s2[i]) {
                if b != 0.0 {
                    out[i] = Some(100.0 * a / b);
                }
            }
        }
    }
    let color = strv(o, "color", "#2962ff");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_donchian(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let hh = highest_arr(&high, len);
    let ll = lowest_arr(&low, len);
    let n = c.len();
    let mut mid = vec![None; n];
    for i in 0..n {
        if let (Some(h), Some(l)) = (hh[i], ll[i]) {
            mid[i] = Some((h + l) / 2.0);
        }
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &hh, &color, SeriesKind::Line, lw),
        build_series(c, &mid, &color, SeriesKind::Line, lw),
        build_series(c, &ll, &color, SeriesKind::Line, lw),
    ]
}

fn compute_stochrsi(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let rsi_len = int(o, "rsiLength", 14).max(1) as usize;
    let stoch_len = int(o, "stochLength", 14).max(1) as usize;
    let k_len = int(o, "k", 3).max(1) as usize;
    let d_len = int(o, "d", 3).max(1) as usize;
    let cl = src_arr(c, "close");
    let n = cl.len();
    // RSI
    let mut rsi = vec![None; n];
    if n > rsi_len {
        let mut gain = 0.0;
        let mut loss = 0.0;
        for i in 1..=rsi_len {
            let d = cl[i] - cl[i - 1];
            if d >= 0.0 {
                gain += d;
            } else {
                loss -= d;
            }
        }
        let mut ag = gain / rsi_len as f64;
        let mut al = loss / rsi_len as f64;
        rsi[rsi_len] = Some(if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) });
        for i in (rsi_len + 1)..n {
            let d = cl[i] - cl[i - 1];
            let g = if d > 0.0 { d } else { 0.0 };
            let l = if d < 0.0 { -d } else { 0.0 };
            ag = (ag * (rsi_len as f64 - 1.0) + g) / rsi_len as f64;
            al = (al * (rsi_len as f64 - 1.0) + l) / rsi_len as f64;
            rsi[i] = Some(if al == 0.0 { 100.0 } else { 100.0 - 100.0 / (1.0 + ag / al) });
        }
    }
    let rsi_filled: Vec<f64> = rsi.iter().map(|v| v.unwrap_or(0.0)).collect();
    let hh = highest_arr(&rsi_filled, stoch_len as i64);
    let ll = lowest_arr(&rsi_filled, stoch_len as i64);
    let mut raw_k = vec![None; n];
    for i in 0..n {
        if rsi[i].is_some() {
            if let (Some(h), Some(l)) = (hh[i], ll[i]) {
                raw_k[i] = Some(if h != l { (rsi_filled[i] - l) / (h - l) * 100.0 } else { 0.0 });
            }
        }
    }
    let raw_k_filled: Vec<f64> = raw_k.iter().map(|v| v.unwrap_or(0.0)).collect();
    let k_arr = sma_arr(&raw_k_filled, k_len as i64);
    let k_filled: Vec<f64> = k_arr.iter().map(|v| v.unwrap_or(0.0)).collect();
    let d_arr = sma_arr(&k_filled, d_len as i64);
    let mut k_out = vec![None; n];
    let mut d_out = vec![None; n];
    for i in 0..n {
        if raw_k[i].is_some() {
            k_out[i] = k_arr[i];
        }
        if k_arr[i].is_some() {
            d_out[i] = d_arr[i];
        }
    }
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &k_out, &strv(o, "kColor", "#2962ff"), SeriesKind::Line, lw),
        build_series(c, &d_out, &strv(o, "dColor", "#ff6d00"), SeriesKind::Line, lw),
    ]
}

fn compute_elderforce(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 13).max(1);
    let n = c.len();
    let mut raw = vec![None; n];
    for i in 1..n {
        raw[i] = Some((c[i].close - c[i - 1].close) * c[i].volume);
    }
    let filled: Vec<f64> = raw.iter().map(|v| v.unwrap_or(0.0)).collect();
    let ema = ema_arr(&filled, len);
    let mut out = vec![None; n];
    for i in 0..n {
        if raw[i].is_some() {
            out[i] = ema[i];
        }
    }
    let color = strv(o, "color", "#7e57c2");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_keltner(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20);
    let mult = num(o, "mult", 2.0);
    let cl = src_arr(c, "close");
    let mid = ema_arr(&cl, len);
    let atr = wilder_arr(&tr_arr(c), len);
    let n = c.len();
    let mut up = vec![None; n];
    let mut lo = vec![None; n];
    for i in 0..n {
        if let (Some(m), Some(a)) = (mid[i], atr[i]) {
            up[i] = Some(m + mult * a);
            lo[i] = Some(m - mult * a);
        }
    }
    let color = strv(o, "color", "#42a5f5");
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &up, &color, SeriesKind::Line, lw),
        build_series(c, &mid, &color, SeriesKind::Line, lw),
        build_series(c, &lo, &color, SeriesKind::Line, lw),
    ]
}

fn compute_chandelier(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 22);
    let mult = num(o, "mult", 3.0);
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let hh = highest_arr(&high, len);
    let ll = lowest_arr(&low, len);
    let atr = wilder_arr(&tr_arr(c), len);
    let n = c.len();
    let mut long = vec![None; n];
    let mut short = vec![None; n];
    for i in 0..n {
        if let (Some(h), Some(l), Some(a)) = (hh[i], ll[i], atr[i]) {
            long[i] = Some(h - mult * a);
            short[i] = Some(l + mult * a);
        }
    }
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &long, &strv(o, "longColor", "#26a69a"), SeriesKind::Line, lw),
        build_series(c, &short, &strv(o, "shortColor", "#ef5350"), SeriesKind::Line, lw),
    ]
}

fn compute_sqzmom(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 20) as usize;
    let mult = num(o, "mult", 2.0);
    let cl = src_arr(c, "close");
    let n = c.len();
    let basis = sma_arr(&cl, len as i64);
    let dev = stdev_arr(&cl, len as i64);
    let atr = wilder_arr(&tr_arr(c), len as i64);
    let mut upper = vec![None; n];
    let mut lower = vec![None; n];
    let mut sqz = vec![false; n];
    for i in 0..n {
        if let (Some(b), Some(d), Some(a)) = (basis[i], dev[i], atr[i]) {
            upper[i] = Some(b + mult * d);
            lower[i] = Some(b - mult * d);
            let bb_up = b + mult * d;
            let bb_lo = b - mult * d;
            let kc_up = b + mult * a;
            let kc_lo = b - mult * a;
            sqz[i] = bb_up < kc_up && bb_lo > kc_lo;
        }
    }
    // momentum = linreg of (close - avg of donchian mid & sma)
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let hh = highest_arr(&high, len as i64);
    let ll = lowest_arr(&low, len as i64);
    let mut out = vec![None; n];
    for i in 0..n {
        if let (Some(b), Some(h), Some(l)) = (basis[i], hh[i], ll[i]) {
            let avg = (h + l) / 2.0;
            let src = cl[i] - (avg + b) / 2.0;
            // linear regression value at end over `len` window
            let start = i + 1 - len;
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_xx = 0.0;
            for (k, j) in (start..=i).enumerate() {
                let x = k as f64;
                let y = cl[j] - {
                    let bj = basis[j].unwrap_or(b);
                    let hj = hh[j].unwrap_or(h);
                    let lj = ll[j].unwrap_or(l);
                    ((hj + lj) / 2.0 + bj) / 2.0
                };
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_xx += x * x;
            }
            let _ = src;
            let denom = len as f64 * sum_xx - sum_x * sum_x;
            if denom != 0.0 {
                let slope = (len as f64 * sum_xy - sum_x * sum_y) / denom;
                let intercept = (sum_y - slope * sum_x) / len as f64;
                out[i] = Some(intercept + slope * (len as f64 - 1.0));
            }
        }
    }
    let _ = sqz;
    let color = strv(o, "color", "#7e57c2");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_fisher(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let len = int(o, "length", 9).max(1) as usize;
    let n = c.len();
    let mut out = vec![None; n];
    let mut value = 0.0f64;
    let mut fish = 0.0f64;
    let mut started = false;
    for i in 0..n {
        if i + 1 < len {
            continue;
        }
        let start = i + 1 - len;
        let mut hi = f64::NEG_INFINITY;
        let mut lo = f64::INFINITY;
        for j in start..=i {
            hi = hi.max((c[j].high + c[j].low) / 2.0);
            lo = lo.min((c[j].high + c[j].low) / 2.0);
        }
        let hl2 = (c[i].high + c[i].low) / 2.0;
        let mut v = if hi != lo { 0.66 * ((hl2 - lo) / (hi - lo) - 0.5) + 0.67 * value } else { 0.0 };
        v = v.clamp(-0.999, 0.999);
        value = v;
        if !started {
            fish = 0.5 * (1.0 + v).ln() + 0.5 * fish;
            started = true;
        } else {
            fish = 0.5 * (1.0 + v).ln() + 0.5 * fish;
        }
        out[i] = Some(fish);
    }
    let color = strv(o, "color", "#2962ff");
    let lw = num(o, "lineWidth", 1.0);
    vec![build_series(c, &out, &color, SeriesKind::Line, lw)]
}

fn compute_ichimoku(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let tenkan = int(o, "tenkan", 9);
    let kijun = int(o, "kijun", 26);
    let senkou = int(o, "senkou", 52);
    let high: Vec<f64> = c.iter().map(|x| x.high).collect();
    let low: Vec<f64> = c.iter().map(|x| x.low).collect();
    let n = c.len();
    let hh_t = highest_arr(&high, tenkan);
    let ll_t = lowest_arr(&low, tenkan);
    let hh_k = highest_arr(&high, kijun);
    let ll_k = lowest_arr(&low, kijun);
    let hh_s = highest_arr(&high, senkou);
    let ll_s = lowest_arr(&low, senkou);
    let mut tenkan_arr = vec![None; n];
    let mut kijun_arr = vec![None; n];
    let mut span_a = vec![None; n];
    let mut span_b = vec![None; n];
    let disp = kijun as usize;
    for i in 0..n {
        if let (Some(h), Some(l)) = (hh_t[i], ll_t[i]) {
            tenkan_arr[i] = Some((h + l) / 2.0);
        }
        if let (Some(h), Some(l)) = (hh_k[i], ll_k[i]) {
            kijun_arr[i] = Some((h + l) / 2.0);
        }
        if i >= disp {
            if let (Some(a), Some(b)) = (tenkan_arr[i - disp], kijun_arr[i - disp]) {
                span_a[i] = Some((a + b) / 2.0);
            }
            if let (Some(h), Some(l)) = (hh_s[i - disp], ll_s[i - disp]) {
                span_b[i] = Some((h + l) / 2.0);
            }
        }
    }
    let lw = num(o, "lineWidth", 1.0);
    vec![
        build_series(c, &tenkan_arr, &strv(o, "tenkanColor", "#2962ff"), SeriesKind::Line, lw),
        build_series(c, &kijun_arr, &strv(o, "kijunColor", "#ef5350"), SeriesKind::Line, lw),
        build_series(c, &span_a, &strv(o, "spanAColor", "#26a69a"), SeriesKind::Line, lw),
        build_series(c, &span_b, &strv(o, "spanBColor", "#ff6d00"), SeriesKind::Line, lw),
    ]
}

/// SMA that ignores NaN warmup values (used by BB%b).
fn sma_arr_tolerant(vals: &[f64], p: usize) -> Vec<Option<f64>> {
    let n = vals.len();
    let mut out = vec![None; n];
    if p == 0 {
        return out;
    }
    let mut sum = 0.0;
    let mut cnt = 0usize;
    for i in 0..n {
        if vals[i].is_finite() {
            sum += vals[i];
            cnt += 1;
        }
        if i >= p {
            if vals[i - p].is_finite() {
                sum -= vals[i - p];
                cnt -= 1;
            }
        }
        if cnt == p {
            out[i] = Some(sum / p as f64);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Advanced structural / overlay indicators (fresh engines)
// ---------------------------------------------------------------------------

fn opt_to_f64(vals: &[Option<f64>]) -> Vec<f64> {
    vals.iter()
        .map(|v| match v {
            Some(x) if x.is_finite() => *x,
            _ => 0.0,
        })
        .collect()
}

fn last_finite(arr: &[Option<f64>]) -> f64 {
    for v in arr.iter().rev() {
        if let Some(x) = v {
            if x.is_finite() {
                return *x;
            }
        }
    }
    0.0
}

fn ew_to_pivots(p: &[EwPivot]) -> Vec<Pivot> {
    p.iter().map(Pivot::from).collect()
}

fn clean_candles(c: &[Candle]) -> Vec<Candle> {
    let mut out = Vec::new();
    let mut prev_t: i64 = 0;
    for x in c {
        if x.time > prev_t
            && x.time as f64 != 0.0
            && x.high.is_finite()
            && x.low.is_finite()
            && x.close.is_finite()
            && x.open.is_finite()
        {
            out.push(*x);
            prev_t = x.time;
        }
    }
    out
}

fn dense_seg(c: &[Candle], a_idx: usize, a_val: f64, b_idx: usize, b_val: f64) -> Vec<Point> {
    let mut d = Vec::new();
    if b_idx < a_idx || c.is_empty() {
        return d;
    }
    if a_idx == b_idx {
        if let Some(x) = c.get(a_idx) {
            d.push(Point { time: x.time, value: a_val, color: None });
        }
        return d;
    }
    let span = (b_idx - a_idx) as f64;
    for i in a_idx..=b_idx {
        if let Some(x) = c.get(i) {
            let f = (i - a_idx) as f64 / span;
            d.push(Point { time: x.time, value: a_val + (b_val - a_val) * f, color: None });
        }
    }
    d
}

fn mk_line(color: &str, lw: f64, data: Vec<Point>, style: Option<i32>) -> SeriesOut {
    let mut s = SeriesOut::line(color, lw);
    s.data = data;
    s.line_style = style;
    s
}

fn live_trend_dir(cc: &[Candle], len: usize) -> Vec<i32> {
    let n = cc.len();
    let mut out = vec![0i32; n];
    if n == 0 {
        return out;
    }
    let closes: Vec<f64> = cc.iter().map(|x| x.close).collect();
    let slow = ema_arr(&closes, len.max(2) as i64);
    let mut last = 0i32;
    for i in 0..n {
        match slow[i] {
            Some(e) if e.is_finite() => {
                if closes[i] > e {
                    last = 1;
                } else if closes[i] < e {
                    last = -1;
                } else if last == 0 {
                    last = 1;
                }
            }
            _ => {
                if i > 0 {
                    last = if closes[i] >= closes[i - 1] { 1 } else { -1 };
                }
            }
        }
        out[i] = last;
    }
    out
}

fn color_points_by_dir(cc: &[Candle], data: &mut [Point], dir: &[i32], up: &str, dn: &str) {
    for p in data.iter_mut() {
        let idx = match cc.binary_search_by_key(&p.time, |x| x.time) {
            Ok(i) => i,
            Err(_) => continue,
        };
        if idx < dir.len() {
            p.color = Some(if dir[idx] >= 0 { up.to_string() } else { dn.to_string() });
        }
    }
}

fn compute_pastruct(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let lw = num(o, "lineWidth", 2.0);
    let s = pa_structure(
        c,
        num(o, "pivotLen", 3.0),
        num(o, "atrLen", 14.0),
        num(o, "atrMult", 0.25),
        boolv(o, "showMarkers", true),
        boolv(o, "markersOnly", false),
        &up,
        &dn,
    );
    let data = if strv(o, "lineMode", "zigzag") == "trail" { s.line } else { s.zig };
    vec![mk_line(&up, lw, data, None)]
}

fn markers_pastruct(c: &[Candle], o: &Settings) -> Vec<Marker> {
    pa_structure(
        c,
        num(o, "pivotLen", 3.0),
        num(o, "atrLen", 14.0),
        num(o, "atrMult", 0.25),
        true,
        boolv(o, "markersOnly", false),
        &strv(o, "upColor", "#26a69a"),
        &strv(o, "downColor", "#ef5350"),
    )
    .markers
}

/// Live-data placeholders: the old app reads window.OITrend / option-chain
/// snapshots for these. In the Rust build the chain feed is not wired yet, so
/// they render the same fixed (empty) series shape and fill in once the feed
/// lands.
fn compute_pcr(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let lw = num(o, "lineWidth", 1.0);
    let mut raw = SeriesOut::line(&strv(o, "pcrColor", "#9e9e9e"), lw);
    raw.point_markers = true;
    raw.last_value_visible = true;
    raw.price_scale_id = Some("pcr".into());
    raw.price_lines = vec![PriceLine {
        price: 1.0,
        color: "#607d8b".into(),
        line_width: 1.0,
        line_style: 2,
        title: "PCR 1.0".into(),
    }];
    let mut fast = SeriesOut::line(&strv(o, "fastColor", "#00d4aa"), lw);
    fast.last_value_visible = false;
    fast.price_scale_id = Some("pcr".into());
    let mut slow = SeriesOut::line(&strv(o, "slowColor", "#ff9800"), lw);
    slow.last_value_visible = false;
    slow.price_scale_id = Some("pcr".into());
    let _ = n;
    vec![raw, fast, slow]
}

fn compute_pcrrail(_c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let cols = ["#ff5252", "#00d4aa", "#b39ddb", "#4fc3f7", "#4fc3f7"];
    cols.iter().map(|col| SeriesOut::line(col, lw)).collect()
}

fn compute_iv(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let lw = num(o, "lineWidth", 2.0);
    let mut iv = SeriesOut::line(&strv(o, "color", "#e040fb"), lw);
    let mut delta = SeriesOut::line(&strv(o, "deltaColor", "#00bcd4"), lw);
    delta.last_value_visible = false;
    let mut vega = SeriesOut::line(&strv(o, "vegaColor", "#ff6d00"), lw);
    vega.last_value_visible = false;
    let _ = n;
    iv.data = Vec::new();
    vec![iv, delta, vega]
}

fn compute_projline(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let hist_color = strv(o, "histColor", "#7ee0ff");
    let proj_color = strv(o, "projColor", "#b388ff");
    let empty = || vec![
        SeriesOut::line("#000000", lw),
        SeriesOut::line(&proj_color, lw),
    ];
    let cc = clean_candles(c);
    let n = cc.len();
    if n < 3 {
        return empty();
    }
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2) as usize;
    let atr_mult = if num(o, "atrMult", 2.0) > 0.0 { num(o, "atrMult", 2.0) } else { 2.0 };
    let min_pct = if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 };
    let fwd = (num(o, "fwd", 30.0).round() as i64).max(1);
    let want_piv = (num(o, "pivots", 3.0).round() as i64).max(2) as usize;
    let mode = strv(o, "mode", "trendline");
    let piv = atr_zigzag(&cc, atr_per, atr_mult, min_pct);
    if piv.len() < 2 {
        return empty();
    }
    let used: Vec<ZigPivot> = if mode == "regression" {
        let start = piv.len().saturating_sub(want_piv);
        piv[start..].to_vec()
    } else {
        let highs: Vec<ZigPivot> = piv.iter().filter(|p| p.is_high).copied().collect();
        let lows: Vec<ZigPivot> = piv.iter().filter(|p| !p.is_high).copied().collect();
        let hh = if highs.len() >= 2 { Some(highs[highs.len() - 1].price > highs[highs.len() - 2].price) } else { None };
        let hl = if lows.len() >= 2 { Some(lows[lows.len() - 1].price > lows[lows.len() - 2].price) } else { None };
        let side = match (hh, hl) {
            (Some(true), Some(true)) => "low",
            (Some(false), Some(false)) => "high",
            _ => if piv[piv.len() - 1].is_high { "high" } else { "low" },
        };
        let arr = if side == "high" { highs } else { lows };
        let start = arr.len().saturating_sub(want_piv);
        let mut u: Vec<ZigPivot> = arr[start..].to_vec();
        if u.len() < 2 {
            let start2 = piv.len().saturating_sub(want_piv);
            u = piv[start2..].to_vec();
        }
        u
    };
    if used.len() < 2 {
        return empty();
    }
    let m = used.len() as f64;
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    for p in &used {
        let x = p.idx as f64;
        sx += x;
        sy += p.price;
        sxx += x * x;
        sxy += x * p.price;
    }
    let den = m * sxx - sx * sx;
    let (a, b) = if den.abs() < 1e-9 {
        let p1 = used[0];
        let p2 = used[used.len() - 1];
        let di = (p2.idx as i64 - p1.idx as i64).max(1) as f64;
        let a = (p2.price - p1.price) / di;
        (a, p2.price - a * p2.idx as f64)
    } else {
        let a = (m * sxy - sx * sy) / den;
        (a, (sy - a * sx) / m)
    };
    let val_at = |idx: f64| a * idx + b;
    let start_idx = used[0].idx;
    let mut hist = Vec::new();
    for i in start_idx..n {
        let v = val_at(i as f64);
        if v.is_finite() {
            hist.push(Point { time: cc[i].time, value: v, color: None });
        }
    }
    let interval = if n >= 3 && cc[n - 1].time - cc[n - 2].time > 0 {
        cc[n - 1].time - cc[n - 2].time
    } else {
        60
    };
    let last_t = cc[n - 1].time;
    let mut fut = vec![Point { time: last_t, value: val_at((n - 1) as f64), color: None }];
    for k in 1..=fwd {
        let v = val_at((n - 1) as f64 + k as f64);
        if v.is_finite() {
            fut.push(Point { time: last_t + k * interval, value: v, color: None });
        }
    }
    let dir = live_trend_dir(&cc, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut s0 = mk_line(&hist_color, lw, hist, None);
    color_points_by_dir(&cc, &mut s0.data, &dir, "#26a69a", "#ef5350");
    vec![s0, mk_line(&proj_color, lw, fut, Some(2))]
}

fn compute_wavefib(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let rg = strv(o, "rangeColor", "#9e9e9e");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if a.c.len() < 3 || a.piv.len() < 2 {
        return vec![SeriesOut::line(&rg, lw)];
    }
    let mut zig: Vec<Point> = Vec::new();
    if boolv(o, "showTrend", true) {
        for k in 0..a.piv.len() {
            let nx = a.piv.get(k + 1).unwrap_or(&a.piv[k]);
            let col = if nx.price >= a.piv[k].price { &up } else { &dn };
            zig.push(Point { time: a.c[a.piv[k].idx].time, value: a.piv[k].price, color: Some(col.clone()) });
        }
        if let (Some(last_z), Some(last_c)) = (zig.last(), a.c.last()) {
            if last_c.time > last_z.time {
                let last_p = &a.piv[a.piv.len() - 1];
                let col = if last_c.close >= last_p.price { &up } else { &dn };
                zig.push(Point { time: last_c.time, value: last_c.close, color: Some(col.clone()) });
            }
        }
    }
    let mut price_lines: Vec<PriceLine> = Vec::new();
    if let Some((a2, b2)) = &a.fib {
        if boolv(o, "showFib", false) || boolv(o, "showExt", false) {
            let lvl = |pct: f64| a2.price + (b2.price - a2.price) * (pct / 100.0);
            let fib_color = strv(o, "fibColor", "#ffd54f");
            let ext_color = strv(o, "fibExtColor", "#ff8a65");
            if boolv(o, "showFib", false) {
                for pct in [23.6, 38.2, 50.0, 61.8, 78.6] {
                    price_lines.push(PriceLine {
                        price: lvl(pct),
                        color: fib_color.clone(),
                        line_width: 1.0,
                        line_style: 3,
                        title: format!("{:.1}%", pct),
                    });
                }
            }
            if boolv(o, "showExt", false) {
                for pct in [100.0, 127.2, 161.8, 200.0] {
                    price_lines.push(PriceLine {
                        price: lvl(pct),
                        color: ext_color.clone(),
                        line_width: 1.0,
                        line_style: 2,
                        title: format!("{:.1}%", pct),
                    });
                }
            }
        }
    }
    let mut s = mk_line(&up, lw, zig, None);
    s.price_lines = price_lines;
    vec![s]
}

fn markers_wavefib(c: &[Candle], o: &Settings) -> Vec<Marker> {
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if !boolv(o, "showLabels", true) {
        return Vec::new();
    }
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    a.labels
        .iter()
        .filter_map(|l| a.c.get(l.idx).map(|x| Marker {
            time: x.time,
            position: if l.is_high { "aboveBar" } else { "belowBar" }.into(),
            color: if l.is_high { dn.clone() } else { up.clone() },
            shape: "circle".into(),
            text: l.text.clone(),
            size: 1.0,
        }))
        .collect()
}

fn score_line(c: &[Candle], state: &[i32], color: &str) -> Vec<SeriesOut> {
    let mut score = 0i64;
    let mut data = Vec::new();
    for (k, st) in state.iter().enumerate() {
        score += *st as i64;
        if let Some(x) = c.get(k) {
            data.push(Point { time: x.time, value: score as f64, color: None });
        }
    }
    vec![mk_line(color, 1.0, data, None)]
}

fn compute_ewtrend(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 6.0), num(o, "minPct", 0.15));
    score_line(&a.c, &a.trend_state, "#ffb300")
}

fn compute_patrend(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let s = pa_structure(
        c,
        num(o, "pivotLen", 10.0),
        num(o, "atrLen", 14.0),
        num(o, "atrMult", 0.25),
        false,
        false,
        "#26a69a",
        "#ef5350",
    );
    score_line(c, &s.trend_state, "#26a69a")
}

fn compute_keylevel(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let cc = clean_candles(c);
    if cc.len() < 10 {
        return vec![SeriesOut::line("#000000", lw)];
    }
    let piv = fractal_pivots(&cc, num(o, "strength", 5.0));
    if piv.len() < 3 {
        return vec![SeriesOut::line("#000000", lw)];
    }
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2);
    let atr = wilder_arr(&tr_arr(&cc), atr_per);
    let atr_last = last_finite(&atr);
    let last_close = cc[cc.len() - 1].close;
    let min_pct = if num(o, "minPct", 0.08) >= 0.0 { num(o, "minPct", 0.08) } else { 0.08 };
    let mut tol = (atr_last * if num(o, "tolMult", 0.6) > 0.0 { num(o, "tolMult", 0.6) } else { 0.6 })
        .max(last_close.abs() * (min_pct / 100.0));
    if !(tol > 0.0) {
        tol = if last_close.abs() * 0.001 != 0.0 { last_close.abs() * 0.001 } else { 1.0 };
    }
    let mut cl = cluster_levels(&piv, tol);
    cl.sort_by(|a, b| b.n.partial_cmp(&a.n).unwrap_or(std::cmp::Ordering::Equal).then(b.last.cmp(&a.last)));
    let want = (num(o, "zones", 5.0).round() as i64).max(1) as usize;
    let mut pmin = f64::INFINITY;
    let mut pmax = f64::NEG_INFINITY;
    for l in &cl {
        if l.price < pmin {
            pmin = l.price;
        }
        if l.price > pmax {
            pmax = l.price;
        }
    }
    let min_sep = (tol * 2.5).max((pmax - pmin) * 0.04);
    let mut shown: Vec<Level> = Vec::new();
    for l in &cl {
        if shown.len() >= want {
            break;
        }
        if shown.iter().all(|p| (p.price - l.price).abs() >= min_sep) {
            shown.push(*l);
        }
    }
    let max_n = shown.first().map(|l| l.n).unwrap_or(1.0).max(1.0);
    let cols = [
        strv(o, "c1", "#2962ff"),
        strv(o, "c2", "#ff9800"),
        strv(o, "c3", "#ef5350"),
        strv(o, "c4", "#26a69a"),
        strv(o, "c5", "#ab47bc"),
    ];
    let i1 = cc.len() - 1;
    let mut out = Vec::new();
    for (k, lvl) in shown.iter().enumerate() {
        let mut i0 = lvl.first.min(i1);
        if i0 >= i1 {
            i0 = i1.saturating_sub(1);
        }
        let data = vec![
            Point { time: cc[i0].time, value: lvl.price, color: None },
            Point { time: cc[i1].time, value: lvl.price, color: None },
        ];
        let mut s = mk_line(&cols[k % cols.len()], lw, data, None);
        s.title = Some(format!("{}%", (lvl.n / max_n * 100.0).round() as i64));
        s.price_line_visible = false;
        out.push(s);
    }
    out
}

fn compute_autotrend(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let empty = || vec![SeriesOut::line("#888888", lw)];
    let cc = clean_candles(c);
    if cc.len() < 12 {
        return empty();
    }
    let piv = fractal_pivots(&cc, num(o, "strength", 5.0));
    if piv.len() < 3 {
        return empty();
    }
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2);
    let atr = wilder_arr(&tr_arr(&cc), atr_per);
    let atr_last = last_finite(&atr);
    let last_close = cc[cc.len() - 1].close;
    let min_pct = if num(o, "minPct", 0.05) >= 0.0 { num(o, "minPct", 0.05) } else { 0.05 };
    let mut tol = (atr_last * if num(o, "tolMult", 0.5) > 0.0 { num(o, "tolMult", 0.5) } else { 0.5 })
        .max(last_close.abs() * (min_pct / 100.0));
    if !(tol > 0.0) {
        tol = if last_close.abs() * 0.001 != 0.0 { last_close.abs() * 0.001 } else { 1.0 };
    }
    let best = match auto_trend_line(&cc, &piv, num(o, "look", 60.0), tol) {
        Some(b) => b,
        None => return empty(),
    };
    let col = if best.is_support {
        strv(o, "upColor", "#26a69a")
    } else {
        strv(o, "downColor", "#ef5350")
    };
    let start = if boolv(o, "fullSpan", true) { 0 } else { best.p1_idx };
    let mut data = Vec::new();
    for i in start..cc.len() {
        let v = best.a * i as f64 + best.b;
        if v.is_finite() {
            data.push(Point { time: cc[i].time, value: v, color: None });
        }
    }
    let dir = live_trend_dir(&cc, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut s = mk_line(&col, lw, data, None);
    color_points_by_dir(&cc, &mut s.data, &dir, &strv(o, "upColor", "#26a69a"), &strv(o, "downColor", "#ef5350"));
    vec![s]
}

fn compute_smiio(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let short = int(o, "shortlen", 13).max(1);
    let long = int(o, "longlen", 25).max(2);
    let sig = int(o, "siglen", 9).max(1);
    let lw = num(o, "lineWidth", 1.0);
    let mut pc = vec![0.0; n];
    let mut ap = vec![0.0; n];
    for i in 1..n {
        pc[i] = c[i].close - c[i - 1].close;
        ap[i] = pc[i].abs();
    }
    let p1 = ema_arr(&pc, short);
    let p2 = ema_arr(&opt_to_f64(&p1), long);
    let a1 = ema_arr(&ap, short);
    let a2 = ema_arr(&opt_to_f64(&a1), long);
    let mut smi: Vec<Opt> = vec![None; n];
    for i in 0..n {
        if let Some(a) = a2[i] {
            if a != 0.0 {
                smi[i] = Some(100.0 * p2[i].unwrap_or(0.0) / a);
            }
        }
    }
    let signal = ema_arr(&opt_to_f64(&smi), sig);
    let mut hist: Vec<Opt> = vec![None; n];
    for i in 0..n {
        if let (Some(s), Some(sg)) = (smi[i], signal[i]) {
            hist[i] = Some(s - sg);
        }
    }
    vec![
        build_series(c, &smi, &strv(o, "color", "#7e57c2"), SeriesKind::Line, lw),
        build_series(c, &signal, &strv(o, "signalColor", "#ff6d00"), SeriesKind::Line, lw),
        build_series(c, &hist, &strv(o, "histColor", "#42a5f5"), SeriesKind::Line, lw),
    ]
}

fn compute_smf(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let look = (num(o, "length", 14.0).round() as i64).max(1) as usize;
    let sig = int(o, "signalLen", 9).max(1);
    let vol_len = (num(o, "volLen", 20.0).round() as i64).max(1) as usize;
    let cap = if num(o, "pulseCap", 3.0) > 0.0 { num(o, "pulseCap", 3.0) } else { 3.0 };
    let lw = num(o, "lineWidth", 1.0);
    let mut clv = vec![0.0; n];
    let mut wgt = vec![0.0; n];
    let mut vol_sum = 0.0;
    for i in 0..n {
        let b = c[i];
        let vol = b.volume;
        vol_sum += vol;
        if i >= vol_len {
            vol_sum -= c[i - vol_len].volume;
        }
        let range = b.high - b.low;
        let cl = if range > 0.0 { ((b.close - b.low) - (b.high - b.close)) / range } else { 0.0 };
        clv[i] = cl;
        let avg = if i + 1 >= vol_len { vol_sum / vol_len as f64 } else { vol_sum / (i + 1) as f64 };
        let pulse = if avg > 0.0 { (vol / avg).min(cap) } else { 1.0 };
        wgt[i] = vol * pulse;
    }
    let mut smf_arr = vec![0.0; n];
    let mut sn = 0.0;
    let mut sd = 0.0;
    for i in 0..n {
        sn += clv[i] * wgt[i];
        sd += wgt[i];
        if i >= look {
            sn -= clv[i - look] * wgt[i - look];
            sd -= wgt[i - look];
        }
        if i + 1 >= look {
            smf_arr[i] = if sd > 0.0 { 100.0 * sn / sd } else { 0.0 };
        }
    }
    let signal = ema_arr(&smf_arr, sig);
    let mut main_data = Vec::new();
    let mut sig_data = Vec::new();
    let mut hist_data = Vec::new();
    let mut prev_h: Option<f64> = None;
    let up_col = strv(o, "histUpColor", "#26a69a");
    let dn_col = strv(o, "histDownColor", "#ef5350");
    for i in (look - 1)..n {
        let v = smf_arr[i];
        if v.is_nan() {
            continue;
        }
        main_data.push(Point { time: c[i].time, value: v, color: None });
        let s = match signal[i] {
            Some(s) if !s.is_nan() => s,
            _ => continue,
        };
        sig_data.push(Point { time: c[i].time, value: s, color: None });
        let h = v - s;
        let up = prev_h.map(|p| h >= p).unwrap_or(h >= 0.0);
        hist_data.push(Point { time: c[i].time, value: h, color: Some(if up { up_col.clone() } else { dn_col.clone() }) });
        prev_h = Some(h);
    }
    let mut hs = SeriesOut::hist(&up_col);
    hs.data = hist_data;
    vec![
        mk_line(&strv(o, "color", "#26c6da"), lw, main_data, None),
        mk_line(&strv(o, "signalColor", "#ff6d00"), lw, sig_data, None),
        hs,
    ]
}

fn compute_autosr(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    if n == 0 {
        return Vec::new();
    }
    let lw = num(o, "lineWidth", 1.0);
    let res_color = strv(o, "resColor", "#26a69a");
    let sup_color = strv(o, "supColor", "#ef5350");
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2);
    let atr_mult = if num(o, "atrMult", 2.0) > 0.0 { num(o, "atrMult", 2.0) } else { 2.0 };
    let min_pct = if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 };
    let atr = wilder_arr(&tr_arr(c), atr_per);
    let th = |i: usize, refp: f64| -> f64 {
        let a = match atr[i] {
            Some(v) if v.is_finite() => v * atr_mult,
            _ => 0.0,
        };
        a.max(refp.abs() * (min_pct / 100.0))
    };
    let mut dir = 1i32;
    let mut ext = c[0].high;
    let mut ext_idx = 0usize;
    let mut last_pivot: Option<(bool, f64, usize)> = None;
    for i in 1..n {
        let t = th(i, c[i].close);
        if dir >= 0 {
            if c[i].high > ext {
                ext = c[i].high;
                ext_idx = i;
            }
            if c[i].low <= ext - t {
                last_pivot = Some((true, ext, ext_idx));
                dir = -1;
                ext = c[i].low;
                ext_idx = i;
            }
        } else {
            if c[i].low < ext {
                ext = c[i].low;
                ext_idx = i;
            }
            if c[i].high >= ext + t {
                last_pivot = Some((false, ext, ext_idx));
                dir = 1;
                ext = c[i].high;
                ext_idx = i;
            }
        }
    }
    let (mut res, mut sup, trend_up) = if dir >= 0 {
        (Some(ext), last_pivot.map(|p| p.1), true)
    } else {
        (last_pivot.map(|p| p.1), Some(ext), false)
    };
    if sup.map(|v| !v.is_finite()).unwrap_or(true) {
        sup = Some(c.iter().map(|x| x.low).fold(f64::INFINITY, f64::min));
    }
    if res.map(|v| !v.is_finite()).unwrap_or(true) {
        res = Some(c.iter().map(|x| x.high).fold(f64::NEG_INFINITY, f64::max));
    }
    let res = res.unwrap();
    let sup = sup.unwrap();
    let fmt = |v: f64| if v < 100.0 { format!("{:.3}", v) } else { format!("{:.2}", v) };
    let mk_out = |price: f64, color: &str, active: bool, tag: &str| -> SeriesOut {
        let w = if active { lw + 1.0 } else { lw };
        let mut s = SeriesOut::line(color, w);
        s.data = c.iter().map(|x| Point { time: x.time, value: price, color: None }).collect();
        s.price_lines = vec![PriceLine {
            price,
            color: color.into(),
            line_width: w,
            line_style: if active { 0 } else { 2 },
            title: format!("{} {}", tag, fmt(price)),
        }];
        s
    };
    let mut out = vec![
        mk_out(res, &res_color, trend_up, "RES"),
        mk_out(sup, &sup_color, !trend_up, "SUP"),
    ];
    if boolv(o, "touchLine", true) && n >= 2 {
        let up = trend_up;
        let level = if up { sup } else { res };
        let mut k: i64 = -1;
        if let Some((is_high, _, idx)) = last_pivot {
            if idx + 1 < n && ((up && !is_high) || (!up && is_high)) {
                k = idx as i64;
            }
        }
        if k < 0 {
            let tol = level.abs() * 0.0005;
            let mut i = n as i64 - 2;
            while i >= 0 {
                if c[i as usize].low - tol <= level && c[i as usize].high + tol >= level {
                    k = i;
                    break;
                }
                i -= 1;
            }
        }
        if k >= 0 && (k as usize) < n - 1 {
            let color = if up { strv(o, "touchUpColor", "#26a69a") } else { strv(o, "touchDownColor", "#ef5350") };
            let data = vec![
                Point { time: c[k as usize].time, value: level, color: None },
                Point { time: c[n - 1].time, value: c[n - 1].close, color: None },
            ];
            let mut s = mk_line(&color, lw + 1.0, data, Some(0));
            s.last_value_visible = false;
            s.price_line_visible = false;
            out.push(s);
        }
    }
    out
}

fn break_markers(cc: &[Candle], piv: &[EwPivot], up: &str, dn: &str, require3: bool) -> Vec<Marker> {
    if piv.len() < 3 && require3 {
        return Vec::new();
    }
    let mut last_h: Option<f64> = None;
    let mut prev_h: Option<f64> = None;
    let mut last_l: Option<f64> = None;
    let mut prev_l: Option<f64> = None;
    let mut trend = 0i32;
    let mut out = Vec::new();
    let mut seen: BTreeMap<i64, bool> = BTreeMap::new();
    for p in piv {
        if p.is_high {
            prev_h = last_h;
            last_h = Some(p.price);
        } else {
            prev_l = last_l;
            last_l = Some(p.price);
        }
        let mut nt = trend;
        if let (Some(lh), Some(ph), Some(ll), Some(pl)) = (last_h, prev_h, last_l, prev_l) {
            if lh > ph && ll > pl {
                nt = 1;
            } else if lh < ph && ll < pl {
                nt = -1;
            }
        }
        if nt != trend && nt != 0 {
            if let Some(x) = cc.get(p.idx) {
                if seen.insert(x.time, true).is_none() {
                    out.push(Marker {
                        time: x.time,
                        position: if nt == 1 { "belowBar" } else { "aboveBar" }.into(),
                        color: if nt == 1 { up.into() } else { dn.into() },
                        shape: if nt == 1 { "arrowUp" } else { "arrowDown" }.into(),
                        text: if trend == 0 { "BOS" } else { "CHoCH" }.into(),
                        size: 1.0,
                    });
                }
            }
        }
        trend = nt;
    }
    out
}

/// Trend-start arrows derived from a line's own direction: `arrowUp` prints on
/// the bar where the line turns from falling/flat to rising, `arrowDown` where
/// it turns from rising/flat to falling. This is the exact direction the AST
/// "Straight Line" trend filters read, so the chart arrow and the arrow
/// detection filter always agree.
fn slope_markers(series: &[SeriesOut], idx: usize, up: &str, dn: &str) -> Vec<Marker> {
    let s = match series.get(idx) {
        Some(s) => s,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    let mut prev = 0i32;
    for i in 1..s.data.len() {
        let d = s.data[i].value - s.data[i - 1].value;
        let dir = if d > 0.0 {
            1
        } else if d < 0.0 {
            -1
        } else {
            prev
        };
        if dir != 0 && dir != prev {
            out.push(Marker {
                time: s.data[i].time,
                position: if dir == 1 { "belowBar" } else { "aboveBar" }.into(),
                color: if dir == 1 { up.into() } else { dn.into() },
                shape: if dir == 1 { "arrowUp" } else { "arrowDown" }.into(),
                text: String::new(),
                size: 1.0,
            });
        }
        prev = dir;
    }
    out
}

// Per-indicator arrow markers for the straight-line / overlay family. Each one
// derives arrows from the very line the AST trend filter reads, so the chart
// arrows and the "Arrow detection" filters share one definition.
fn markers_vlcore(c: &[Candle], o: &Settings) -> Vec<Marker> {
    slope_markers(&compute_vlcore(c, o), 0, &strv(o, "upColor", "#00e676"), &strv(o, "downColor", "#ff5252"))
}
fn markers_vl(c: &[Candle], o: &Settings) -> Vec<Marker> {
    slope_markers(&compute_vl(c, o), 0, &strv(o, "color", "#26c6da"), &strv(o, "signalColor", "#ff6d00"))
}
// Straight-line fits (Auto Trendline, Pitchfork, Trend Projection, Gann/Fib Fan,
// S/R EMA Reversal) have a constant slope, so a per-bar slope change never fires
// and the chart shows no arrows. Derive their trend-start arrows from the ATR
// zigzag structure instead - an up arrow where the structure turns to
// higher-highs + higher-lows, a down arrow where it turns lower. This is the
// same structure the trendline is anchored to.
fn markers_pivot_trend(c: &[Candle], o: &Settings, up: &str, dn: &str) -> Vec<Marker> {
    let cc = clean_candles(c);
    let piv = fractal_pivots(&cc, num(o, "strength", 5.0));
    structural_markers(&cc, &piv, up, dn)
}

fn structural_markers(cc: &[Candle], piv: &[Pivot], up: &str, dn: &str) -> Vec<Marker> {
    // Emit an arrow every time the swing structure confirms a direction: an
    // up swing makes a higher-high together with a higher-low, a down swing a
    // lower-high together with a lower-low. Unlike a regime-change detector
    // this keeps firing while a trend runs, so a fresh arrow is always near the
    // right edge and the "Arrow detection" filter has something to trigger on.
    let mut last_h: Option<f64> = None;
    let mut prev_h: Option<f64> = None;
    let mut last_l: Option<f64> = None;
    let mut prev_l: Option<f64> = None;
    let mut out = Vec::new();
    let mut seen: BTreeMap<i64, bool> = BTreeMap::new();
    for p in piv {
        if p.is_high {
            prev_h = last_h;
            last_h = Some(p.price);
        } else {
            prev_l = last_l;
            last_l = Some(p.price);
        }
        let dir = match (last_h, prev_h, last_l, prev_l) {
            (Some(lh), Some(ph), Some(ll), Some(pl)) => {
                if lh > ph && ll > pl {
                    1
                } else if lh < ph && ll < pl {
                    -1
                } else {
                    0
                }
            }
            _ => 0,
        };
        if dir != 0 {
            if let Some(x) = cc.get(p.idx) {
                if seen.insert(x.time, true).is_none() {
                    out.push(Marker {
                        time: x.time,
                        position: if dir == 1 { "belowBar" } else { "aboveBar" }.into(),
                        color: if dir == 1 { up.into() } else { dn.into() },
                        shape: if dir == 1 { "arrowUp" } else { "arrowDown" }.into(),
                        text: "".into(),
                        size: 1.0,
                    });
                }
            }
        }
    }
    out
}
fn markers_srema(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, &strv(o, "bullColor", "#26a69a"), &strv(o, "bearColor", "#ef5350"))
}
fn markers_autotrend(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, &strv(o, "upColor", "#26a69a"), &strv(o, "downColor", "#ef5350"))
}
fn markers_pitchfork(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, "#26a69a", "#ef5350")
}
fn markers_projline(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, "#26a69a", "#ef5350")
}
fn markers_fibfan(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, "#26a69a", "#ef5350")
}
fn markers_gannfan(c: &[Candle], o: &Settings) -> Vec<Marker> {
    markers_pivot_trend(c, o, "#26a69a", "#ef5350")
}
fn markers_supplydemand(c: &[Candle], o: &Settings) -> Vec<Marker> {
    slope_markers(&compute_supplydemand(c, o), 0, "#26a69a", "#ef5350")
}

/// Gate helper for the straight-line arrow filters: did the zigzag structure
/// turn to the requested side exactly at the bar `offset` bars back? Mirrors
/// `markers_pivot_trend`, so the arrow a trader sees on the chart is the arrow
/// this filter detects. `None` when the structure is not resolvable yet
/// (warm-up / fewer than three pivots), treated as non-blocking by the gate.
pub fn trend_flip_at(
    candles: &[Candle],
    offset: usize,
    bull: bool,
    strength: f64,
) -> Option<bool> {
    let n = candles.len();
    if offset >= n {
        return None;
    }
    let target = candles[n - 1 - offset].time;
    let cc = clean_candles(candles);
    let piv = fractal_pivots(&cc, strength);
    if piv.len() < 3 {
        return None;
    }
    let mk = structural_markers(&cc, &piv, "#26a69a", "#ef5350");
    let want = if bull { "arrowUp" } else { "arrowDown" };
    Some(mk.iter().any(|m| m.time == target && m.shape == want))
}

fn zig_points(cc: &[Candle], piv: &[EwPivot], up: &str, dn: &str) -> Vec<Point> {
    let mut zig: Vec<Point> = Vec::new();
    for k in 0..piv.len() {
        let nx = piv.get(k + 1).unwrap_or(&piv[k]);
        let col = if nx.price >= piv[k].price { up } else { dn };
        zig.push(Point { time: cc[piv[k].idx].time, value: piv[k].price, color: Some(col.into()) });
    }
    if let (Some(last_z), Some(last_c)) = (zig.last(), cc.last()) {
        if last_c.time > last_z.time {
            let last_p = &piv[piv.len() - 1];
            let col = if last_c.close >= last_p.price { up } else { dn };
            zig.push(Point { time: last_c.time, value: last_c.close, color: Some(col.into()) });
        }
    }
    zig
}

fn compute_zzline(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if a.c.len() < 3 || a.piv.len() < 2 {
        return vec![SeriesOut::line(&up, lw), SeriesOut::line(&strv(o, "trendColor", "#2962ff"), lw)];
    }
    let zig = if boolv(o, "showZig", true) { zig_points(&a.c, &a.piv, &up, &dn) } else { Vec::new() };
    let mut line = Vec::new();
    let mut line_color = strv(o, "trendColor", "#2962ff");
    if boolv(o, "showLine", true) {
        let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2);
        let atr = wilder_arr(&tr_arr(&a.c), atr_per);
        let atr_last = last_finite(&atr);
        let last_close = a.c[a.c.len() - 1].close;
        let min_pct = if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 };
        let mut tol = (atr_last * 0.5).max(last_close.abs() * (min_pct / 100.0));
        if !(tol > 0.0) {
            tol = if last_close.abs() * 0.001 != 0.0 { last_close.abs() * 0.001 } else { 1.0 };
        }
        if let Some(best) = auto_trend_line(&a.c, &ew_to_pivots(&a.piv), num(o, "pivotLook", 8.0), tol) {
            line_color = if best.is_support { up.clone() } else { dn.clone() };
            let start = if boolv(o, "fullSpan", true) { 0 } else { best.p1_idx };
            for i in start..a.c.len() {
                let v = best.a * i as f64 + best.b;
                if v.is_finite() {
                    line.push(Point { time: a.c[i].time, value: v, color: None });
                }
            }
        }
    }
    let dir = live_trend_dir(&a.c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut s0 = mk_line(&up, lw, zig, None);
    let mut s1 = mk_line(&line_color, lw, line, None);
    color_points_by_dir(&a.c, &mut s0.data, &dir, &up, &dn);
    color_points_by_dir(&a.c, &mut s1.data, &dir, &up, &dn);
    vec![s0, s1]
}

fn markers_zzline(c: &[Candle], o: &Settings) -> Vec<Marker> {
    if !boolv(o, "showBreaks", true) {
        return Vec::new();
    }
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    break_markers(&a.c, &a.piv, &strv(o, "upColor", "#26a69a"), &strv(o, "downColor", "#ef5350"), false)
}

fn compute_trendmaster(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let level_color = strv(o, "levelColor", "#ffb300");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if a.c.len() < 3 || a.piv.len() < 2 {
        return vec![
            SeriesOut::line(&up, lw),
            SeriesOut::line(&strv(o, "trendColor", "#2962ff"), lw),
            SeriesOut::line(&level_color, lw),
        ];
    }
    let zig = if !boolv(o, "lineOnly", false) { zig_points(&a.c, &a.piv, &up, &dn) } else { Vec::new() };
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2);
    let atr = wilder_arr(&tr_arr(&a.c), atr_per);
    let atr_last = last_finite(&atr);
    let last_close = a.c[a.c.len() - 1].close;
    let min_pct = if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 };
    let mut tol = (atr_last * 0.5).max(last_close.abs() * (min_pct / 100.0));
    if !(tol > 0.0) {
        tol = if last_close.abs() * 0.001 != 0.0 { last_close.abs() * 0.001 } else { 1.0 };
    }
    let mut line = Vec::new();
    let mut line_color = strv(o, "trendColor", "#2962ff");
    if let Some(best) = auto_trend_line(&a.c, &ew_to_pivots(&a.piv), num(o, "trendLook", 12.0), tol) {
        line_color = if best.is_support { up.clone() } else { dn.clone() };
        let start = if boolv(o, "fullSpan", true) { 0 } else { best.p1_idx };
        for i in start..a.c.len() {
            let v = best.a * i as f64 + best.b;
            if v.is_finite() {
                line.push(Point { time: a.c[i].time, value: v, color: None });
            }
        }
    }
    let mut level = Vec::new();
    if boolv(o, "showLevel", false) {
        let lvtol = if atr_last > 0.0 { atr_last } else if last_close.abs() * (min_pct / 100.0) != 0.0 { last_close.abs() * (min_pct / 100.0) } else { 1.0 };
        let lv = cluster_levels(&ew_to_pivots(&a.piv), lvtol);
        if !lv.is_empty() {
            let mut strong = lv[0];
            for l in lv.iter().skip(1) {
                if l.n > strong.n {
                    strong = *l;
                }
            }
            level = a.c.iter().map(|x| Point { time: x.time, value: strong.price, color: None }).collect();
        }
    }
    let mut ls = mk_line(&level_color, lw, level, Some(2));
    ls.last_value_visible = false;
    ls.price_line_visible = false;
    let dir = live_trend_dir(&a.c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut s0 = mk_line(&up, lw, zig, None);
    let mut s1 = mk_line(&line_color, lw, line, None);
    color_points_by_dir(&a.c, &mut s0.data, &dir, &up, &dn);
    color_points_by_dir(&a.c, &mut s1.data, &dir, &up, &dn);
    vec![s0, s1, ls]
}

fn markers_trendmaster(c: &[Candle], o: &Settings) -> Vec<Marker> {
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if a.c.len() < 3 || a.piv.len() < 2 {
        return Vec::new();
    }
    let mut mk: Vec<Marker> = Vec::new();
    let mut seen: BTreeMap<i64, bool> = BTreeMap::new();
    if boolv(o, "showBreaks", true) {
        mk = break_markers(&a.c, &a.piv, &up, &dn, true);
        for m in &mk {
            seen.insert(m.time, true);
        }
    }
    if boolv(o, "showVol", true) && !a.c.is_empty() {
        let vlen = (num(o, "volLength", 21.0).round() as i64).max(2) as usize;
        let vmult = if num(o, "volMult", 1.5) > 0.0 { num(o, "volMult", 1.5) } else { 1.5 };
        let vols: Vec<f64> = a.c.iter().map(|x| if x.volume.is_finite() { x.volume } else { 0.0 }).collect();
        let mut bias = vec![0i32; a.c.len()];
        let mut b = 0i32;
        let mut pi = 0usize;
        let (mut lh, mut ph, mut ll, mut pl): (Option<f64>, Option<f64>, Option<f64>, Option<f64>) = (None, None, None, None);
        for i in 0..a.c.len() {
            while pi < a.piv.len() && a.piv[pi].at <= i {
                let p = &a.piv[pi];
                pi += 1;
                if p.is_high {
                    ph = lh;
                    lh = Some(p.price);
                } else {
                    pl = ll;
                    ll = Some(p.price);
                }
                if let (Some(a1), Some(b1), Some(c1), Some(d1)) = (lh, ph, ll, pl) {
                    if a1 > b1 && c1 > d1 {
                        b = 1;
                    } else if a1 < b1 && c1 < d1 {
                        b = -1;
                    }
                }
            }
            bias[i] = b;
        }
        let vol_color = strv(o, "volColor", "#ffd54f");
        let mut vsum = 0.0;
        for i in 0..a.c.len() {
            if i >= vlen + 1 {
                vsum -= vols[i - vlen - 1];
            }
            if i >= 1 {
                vsum += vols[i - 1];
            }
            let cnt = i.min(vlen);
            if cnt < vlen {
                continue;
            }
            let avg = vsum / cnt as f64;
            if bias[i] == 0 || avg <= 0.0 || !(vols[i] > vmult * avg) {
                continue;
            }
            let body_up = a.c[i].close >= a.c[i].open;
            if (bias[i] == 1) != body_up {
                continue;
            }
            let t = a.c[i].time;
            if seen.insert(t, true).is_some() {
                continue;
            }
            mk.push(Marker {
                time: t,
                position: "inBar".into(),
                color: vol_color.clone(),
                shape: "circle".into(),
                text: String::new(),
                size: 1.0,
            });
        }
    }
    mk
}

fn compute_panemaster(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let trend = strv(o, "trendColor", "#2962ff");
    let sig = pane_signal(c, o);
    if sig.cc.is_empty() {
        return vec![SeriesOut::line(&trend, lw), SeriesOut::line(&trend, lw)];
    }
    let fit_color = if sig.cur_reg > 0 { up.clone() } else { dn.clone() };
    let dir = live_trend_dir(&sig.cc, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut s0 = mk_line(&up, lw, sig.data.clone(), None);
    color_points_by_dir(&sig.cc, &mut s0.data, &dir, &up, &dn);
    let mut fit = mk_line(&fit_color, lw, sig.fit_data.clone(), Some(2));
    color_points_by_dir(&sig.cc, &mut fit.data, &dir, &up, &dn);
    fit.last_value_visible = false;
    fit.price_line_visible = false;
    vec![s0, fit]
}

fn markers_panemaster(c: &[Candle], o: &Settings) -> Vec<Marker> {
    let sig = pane_signal(c, o);
    if sig.cc.len() < 35 {
        return Vec::new();
    }
    let up = strv(o, "upColor", "#26a69a");
    let dn = strv(o, "downColor", "#ef5350");
    let mut mk: Vec<Marker> = Vec::new();
    let mut seen: BTreeMap<i64, bool> = BTreeMap::new();
    for i in 1..sig.cc.len() {
        if sig.regime[i] != sig.regime[i - 1] && sig.regime[i] != 0 {
            let t = sig.cc[i].time;
            if seen.insert(t, true).is_none() {
                let bull = sig.regime[i] > 0;
                mk.push(Marker {
                    time: t,
                    position: if bull { "belowBar" } else { "aboveBar" }.into(),
                    color: if bull { up.clone() } else { dn.clone() },
                    shape: if bull { "arrowUp" } else { "arrowDown" }.into(),
                    text: if bull { "BULL" } else { "BEAR" }.into(),
                    size: 1.0,
                });
            }
        }
    }
    if boolv(o, "showExhaustion", true) {
        let ex = strv(o, "exhaustColor", "#ffb300");
        let (mut prev_ob, mut prev_os) = (false, false);
        for i in 0..sig.cc.len() {
            if sig.regime[i] == 0 {
                prev_ob = false;
                prev_os = false;
                continue;
            }
            let mut ob = 0;
            let mut os = 0;
            if let Some(Some(a)) = sig.rsi.get(i).copied() {
                if a > 70.0 { ob += 1; } else if a < 30.0 { os += 1; }
            }
            if let Some(Some(b)) = sig.bb_pct.get(i).copied() {
                if b > 1.0 { ob += 1; } else if b < 0.0 { os += 1; }
            }
            if let Some(Some(k)) = sig.st_k.get(i).copied() {
                if k > 80.0 { ob += 1; } else if k < 20.0 { os += 1; }
            }
            if let Some(Some(cc)) = sig.cci.get(i).copied() {
                if cc > 100.0 { ob += 1; } else if cc < -100.0 { os += 1; }
            }
            if let Some(Some(w)) = sig.will_r.get(i).copied() {
                if w > -20.0 { ob += 1; } else if w < -80.0 { os += 1; }
            }
            if let Some(Some(mf)) = sig.mfi.get(i).copied() {
                if mf > 80.0 { ob += 1; } else if mf < 20.0 { os += 1; }
            }
            let ob_now = sig.regime[i] > 0 && ob >= 4;
            let os_now = sig.regime[i] < 0 && os >= 4;
            let t = sig.cc[i].time;
            if ob_now && !prev_ob && seen.insert(t, true).is_none() {
                mk.push(Marker { time: t, position: "aboveBar".into(), color: ex.clone(), shape: "circle".into(), text: "OB".into(), size: 1.0 });
            } else if os_now && !prev_os && seen.insert(t, true).is_none() {
                mk.push(Marker { time: t, position: "belowBar".into(), color: ex.clone(), shape: "circle".into(), text: "OS".into(), size: 1.0 });
            }
            prev_ob = ob_now;
            prev_os = os_now;
        }
    }
    mk
}

fn compute_pitchfork(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let med = strv(o, "medianColor", "#2962ff");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    if a.c.len() < 3 {
        return vec![SeriesOut::line(&med, lw), SeriesOut::line(&med, lw), SeriesOut::line(&med, lw)];
    }
    let min_span = (num(o, "minSpan", 12.0).round() as i64).max(3) as f64;
    let mut rng = 0.0;
    {
        let hi = a.c.iter().map(|x| x.high).fold(f64::NEG_INFINITY, f64::max);
        let lo = a.c.iter().map(|x| x.low).fold(f64::INFINITY, f64::min);
        rng = hi - lo;
    }
    let mut sel: Option<(EwPivot, EwPivot, EwPivot, f64)> = None;
    let piv = &a.piv;
    if piv.len() >= 3 {
        let mut end = piv.len() as i64 - 1;
        while end >= 2 {
            let c3 = &piv[end as usize];
            let b2 = &piv[(end - 1) as usize];
            let a1 = &piv[(end - 2) as usize];
            if a1.is_high == b2.is_high || b2.is_high == c3.is_high {
                end -= 1;
                continue;
            }
            if !(a1.idx < b2.idx && b2.idx < c3.idx) {
                end -= 1;
                continue;
            }
            let span = c3.idx as f64 - a1.idx as f64;
            if span < min_span {
                end -= 1;
                continue;
            }
            let mx0 = (a1.idx + c3.idx) as f64 / 2.0;
            if (mx0 - b2.idx as f64).abs() < (2.0f64).max(span * 0.15) {
                end -= 1;
                continue;
            }
            let my0 = (a1.price + c3.price) / 2.0;
            let sl = (my0 - b2.price) / (mx0 - b2.idx as f64);
            if !sl.is_finite() || sl.abs() * span > rng * 0.8 {
                end -= 1;
                continue;
            }
            sel = Some((a1.clone(), b2.clone(), c3.clone(), sl));
            break;
        }
    }
    let (a1, b2, c3, slope) = match sel {
        Some(v) => v,
        None => return vec![SeriesOut::line(&med, lw), SeriesOut::line(&med, lw), SeriesOut::line(&med, lw)],
    };
    let start_idx = if boolv(o, "fullSpan", false) { 0 } else { a1.idx.min(b2.idx).min(c3.idx) };
    let mk = |base: &EwPivot| -> Vec<Point> {
        let mut d = Vec::new();
        for i in start_idx..a.c.len() {
            let v = base.price + slope * (i as f64 - base.idx as f64);
            if v.is_finite() {
                d.push(Point { time: a.c[i].time, value: v, color: None });
            }
        }
        d
    };
    let mut s0 = mk_line(&med, lw, mk(&b2), None);
    s0.exclude_autoscale = true;
    let mut s1 = mk_line(&strv(o, "upperColor", "#ef5350"), lw, mk(&a1), None);
    s1.exclude_autoscale = true;
    let mut s2 = mk_line(&strv(o, "lowerColor", "#26a69a"), lw, mk(&c3), None);
    s2.exclude_autoscale = true;
    let dir = live_trend_dir(&a.c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    color_points_by_dir(&a.c, &mut s0.data, &dir, &strv(o, "lowerColor", "#26a69a"), &strv(o, "upperColor", "#ef5350"));
    vec![s0, s1, s2]
}

fn fan_rays(c: &[Candle], o: &Settings, ratios: &[f64], color_key: &str, default_color: &str) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let col = strv(o, color_key, default_color);
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    let empty = || ratios.iter().map(|_| SeriesOut::line(&col, lw)).collect::<Vec<_>>();
    if a.c.len() < 3 {
        return empty();
    }
    let t = match last_alternating_pivots(&ew_to_pivots(&a.piv), 2) {
        Some(v) => v,
        None => return empty(),
    };
    let p0 = t[0];
    let p1 = t[1];
    let dx = p1.idx as i64 - p0.idx as i64;
    if dx <= 0 {
        return empty();
    }
    let dy = p1.price - p0.price;
    let dir = live_trend_dir(&a.c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut out: Vec<SeriesOut> = ratios
        .iter()
        .map(|r| {
            let slope = (r * dy) / dx as f64;
            let mut d = Vec::new();
            for i in p0.idx..a.c.len() {
                let v = p0.price + slope * (i as f64 - p0.idx as f64);
                if v.is_finite() {
                    d.push(Point { time: a.c[i].time, value: v, color: None });
                }
            }
            let mut s = mk_line(&col, lw, d, None);
            s.exclude_autoscale = true;
            s
        })
        .collect();
    for s in out.iter_mut() {
        color_points_by_dir(&a.c, &mut s.data, &dir, "#26a69a", "#ef5350");
    }
    out
}

fn compute_fibfan(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    fan_rays(c, o, &[0.236, 0.382, 0.5, 0.618, 0.786], "fanColor", "#ab47bc")
}

fn compute_gannfan(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let mults = [8.0, 4.0, 3.0, 2.0, 1.0, 0.5, 1.0 / 3.0, 0.25, 0.125];
    let lw = num(o, "lineWidth", 1.0);
    let col = strv(o, "fanColor", "#607d8b");
    let a = ew_analyze(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15));
    let empty = || mults.iter().map(|_| SeriesOut::line(&col, lw)).collect::<Vec<_>>();
    if a.c.len() < 3 {
        return empty();
    }
    let t = match last_alternating_pivots(&ew_to_pivots(&a.piv), 2) {
        Some(v) => v,
        None => return empty(),
    };
    let p0 = t[0];
    let p1 = t[1];
    let dx = p1.idx as i64 - p0.idx as i64;
    if dx <= 0 {
        return empty();
    }
    let base = (p1.price - p0.price) / dx as f64;
    let dir = live_trend_dir(&a.c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut out: Vec<SeriesOut> = mults
        .iter()
        .map(|m| {
            let slope = m * base;
            let mut d = Vec::new();
            for i in p0.idx..a.c.len() {
                let v = p0.price + slope * (i as f64 - p0.idx as f64);
                if v.is_finite() {
                    d.push(Point { time: a.c[i].time, value: v, color: None });
                }
            }
            let mut s = mk_line(&col, lw, d, None);
            s.exclude_autoscale = true;
            s
        })
        .collect();
    for s in out.iter_mut() {
        color_points_by_dir(&a.c, &mut s.data, &dir, "#26a69a", "#ef5350");
    }
    out
}

fn compute_fibt(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    fan_swing_trend(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15), "#ab47bc")
}

fn compute_gant(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    fan_swing_trend(c, num(o, "atrPeriod", 14.0), num(o, "atrMult", 2.0), num(o, "minPct", 0.15), "#607d8b")
}

fn compute_srema(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 2.0);
    let bull = strv(o, "bullColor", "#26a69a");
    let bear = strv(o, "bearColor", "#ef5350");
    let a = srema_analyze(
        c,
        num(o, "atrPeriod", 14.0),
        num(o, "atrMult", 2.0),
        num(o, "minPct", 0.15),
        num(o, "emaPeriod", 1.0),
        num(o, "touchMult", 0.5),
        num(o, "touchWindow", 3.0),
        num(o, "fwd", 20.0),
    );
    if a.c.len() < 3 {
        return vec![SeriesOut::line(&bull, lw), SeriesOut::line(&bear, lw)];
    }
    let cc = &a.c;
    let n = cc.len();
    let dir = live_trend_dir(cc, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let want_bull = dir[n - 1] >= 0;
    let piv = atr_zigzag(
        cc,
        (num(o, "atrPeriod", 14.0).round() as i64).max(2) as usize,
        if num(o, "atrMult", 2.0) > 0.0 { num(o, "atrMult", 2.0) } else { 2.0 },
        if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 },
    );
    let mut anchor: Option<(usize, f64)> = None;
    for p in piv.iter().rev() {
        if p.idx < n && ((want_bull && !p.is_high) || (!want_bull && p.is_high)) {
            anchor = Some((p.idx, p.price));
            break;
        }
    }
    let mut data = Vec::new();
    if let Some((ai, ap)) = anchor {
        let span = (n - 1).saturating_sub(ai);
        if span > 0 {
            let slope = (cc[n - 1].close - ap) / span as f64;
            for i in ai..n {
                let v = ap + slope * (i - ai) as f64;
                if v.is_finite() {
                    data.push(Point {
                        time: cc[i].time,
                        value: v,
                        color: Some((if want_bull { &bull } else { &bear }).clone()),
                    });
                }
            }
        }
    }
    let mut s1 = SeriesOut::line(&bull, lw);
    let mut s2 = SeriesOut::line(&bear, lw);
    s1.last_value_visible = false;
    s1.price_line_visible = false;
    s2.last_value_visible = false;
    s2.price_line_visible = false;
    if want_bull {
        s1.data = data;
    } else {
        s2.data = data;
    }
    vec![s1, s2]
}

fn compute_sremat(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let a = srema_analyze(
        c,
        num(o, "atrPeriod", 14.0),
        num(o, "atrMult", 2.0),
        num(o, "minPct", 0.15),
        num(o, "emaPeriod", 1.0),
        num(o, "touchMult", 0.5),
        num(o, "touchWindow", 3.0),
        num(o, "fwd", 20.0),
    );
    if a.c.len() < 3 {
        return vec![SeriesOut::line("#ffb300", 1.0)];
    }
    score_line(&a.c, &a.state, "#ffb300")
}

fn compute_supplydemand(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let struct_color = strv(o, "structColor", "#b388ff");
    let live_color = strv(o, "liveColor", "#7ee0ff");
    let proj_color = strv(o, "projColor", "#ffb74d");
    let empty = || SeriesOut::line("#000000", lw);
    let cc = clean_candles(c);
    let n = cc.len();
    if n == 0 {
        return Vec::new();
    }
    let atr_per = (num(o, "atrPeriod", 14.0).round() as i64).max(2) as usize;
    let atr_mult = if num(o, "atrMult", 2.0) > 0.0 { num(o, "atrMult", 2.0) } else { 2.0 };
    let min_pct = if num(o, "minPct", 0.15) >= 0.0 { num(o, "minPct", 0.15) } else { 0.15 };
    let eq_tol = (if num(o, "eqTol", 25.0) >= 0.0 { num(o, "eqTol", 25.0) } else { 25.0 }) / 100.0;
    let piv = atr_zigzag(&cc, atr_per, atr_mult, min_pct);
    if piv.len() < 2 {
        return vec![empty(), empty(), empty()];
    }
    let mut legs: Vec<(bool, f64, ZigPivot, ZigPivot)> = Vec::new();
    for i in 1..piv.len() {
        let a = piv[i - 1];
        let b = piv[i];
        legs.push((b.is_high, (b.price - a.price).abs(), a, b));
    }
    let cap = if piv.len() > 400 { piv.len() - 400 } else { 0 };
    let cap_piv = &piv[cap..];
    let mut s0: Vec<Point> = Vec::new();
    for k in 1..cap_piv.len() {
        let a = cap_piv[k - 1];
        let b = cap_piv[k];
        let mut seg = dense_seg(&cc, a.idx, a.price, b.idx, b.price);
        if k > 1 && !seg.is_empty() {
            seg.remove(0);
        }
        s0.extend(seg);
    }
    let last_p = piv[piv.len() - 1];
    let cur = cc[n - 1];
    let s1 = dense_seg(&cc, last_p.idx, last_p.price, n - 1, cur.close);
    let last_leg = legs.last().copied();
    let mut mirror_ok = false;
    if let Some(ll) = last_leg {
        if legs.len() >= 2 {
            let prev = legs[legs.len() - 2];
            let big = prev.1.max(ll.1).max(1.0);
            mirror_ok = ((prev.1 - ll.1).abs() / big) <= eq_tol;
        } else {
            mirror_ok = true;
        }
    }
    let s2 = if mirror_ok {
        if let Some(ll) = last_leg {
            let want_up = !ll.0;
            let guess = if legs.len() >= 2 { legs[legs.len() - 2].1 } else { ll.1 };
            let target = if want_up { last_p.price + guess } else { last_p.price - guess };
            if target.is_finite() {
                mk_line(&proj_color, lw, dense_seg(&cc, last_p.idx, last_p.price, n - 1, target), Some(2))
            } else {
                empty()
            }
        } else {
            empty()
        }
    } else {
        empty()
    };
    let dir = live_trend_dir(&cc, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut l0 = mk_line(&struct_color, lw, s0, None);
    let mut l1 = mk_line(&live_color, lw, s1, None);
    color_points_by_dir(&cc, &mut l0.data, &dir, "#26a69a", "#ef5350");
    color_points_by_dir(&cc, &mut l1.data, &dir, "#26a69a", "#ef5350");
    vec![l0, l1, s2]
}

fn compute_vl(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let n = c.len();
    let l = (num(o, "length", 14.0).round() as i64).max(1) as usize;
    let sig = int(o, "signalLen", 9).max(1);
    let vol_len = (num(o, "volLen", 20.0).round() as i64).max(1) as usize;
    let lw = num(o, "lineWidth", 1.0);
    let base_color = strv(o, "color", "#26c6da");
    let mut vol_avg = vec![0.0; n];
    let mut v_sum = 0.0;
    for i in 0..n {
        let vol = c[i].volume;
        v_sum += vol;
        if i >= vol_len {
            v_sum -= c[i - vol_len].volume;
        }
        if i + 1 >= vol_len {
            vol_avg[i] = v_sum / vol_len as f64;
        }
    }
    let mut vw = vec![0.0; n];
    let mut pv = 0.0;
    let mut vsum = 0.0;
    for i in 0..n {
        let vol = c[i].volume;
        pv += c[i].close * vol;
        vsum += vol;
        if i >= l {
            let oo = c[i - l];
            let ov = oo.volume;
            pv -= oo.close * ov;
            vsum -= ov;
        }
        if i + 1 >= l && vsum > 0.0 {
            vw[i] = pv / vsum;
        }
    }
    let signal = ema_arr(&vw, sig);
    let hx = base_color.trim_start_matches('#').to_string();
    let hx = if hx.len() == 3 { hx.chars().flat_map(|ch| [ch, ch]).collect::<String>() } else { hx };
    let (cr, cg, cb) = if hx.len() >= 6 {
        (
            u8::from_str_radix(&hx[0..2], 16).unwrap_or(0),
            u8::from_str_radix(&hx[2..4], 16).unwrap_or(0),
            u8::from_str_radix(&hx[4..6], 16).unwrap_or(0),
        )
    } else {
        (0x26, 0xc6, 0xda)
    };
    let mut main_data = Vec::new();
    let mut sig_data = Vec::new();
    for i in (l - 1)..n {
        let v = vw[i];
        if v == 0.0 && i + 1 < l {
            continue;
        }
        let vol = c[i].volume;
        let avg = if vol_avg[i] > 0.0 { vol_avg[i] } else { vol };
        let ratio = if avg > 0.0 { vol / avg } else { 1.0 };
        let mut alpha = 0.18 + ratio * 0.82;
        if alpha < 0.15 {
            alpha = 0.15;
        } else if alpha > 1.0 {
            alpha = 1.0;
        }
        main_data.push(Point {
            time: c[i].time,
            value: v,
            color: Some(format!("rgba({},{},{},{})", cr, cg, cb, alpha)),
        });
        if let Some(sv) = signal[i] {
            if !sv.is_nan() {
                sig_data.push(Point { time: c[i].time, value: sv, color: None });
            }
        }
    }
    if boolv(o, "straight", true) {
        main_data = straighten_line(&main_data, num(o, "straightTol", 0.08));
        sig_data = straighten_line(&sig_data, num(o, "straightTol", 0.08));
    }
    let dir = live_trend_dir(c, num(o, "trendLen", 9.0).round().max(2.0) as usize);
    let mut m0 = mk_line(&base_color, lw, main_data, None);
    let mut m1 = mk_line(&strv(o, "signalColor", "#ff6d00"), lw, sig_data, None);
    color_points_by_dir(c, &mut m0.data, &dir, "#26a69a", "#ef5350");
    color_points_by_dir(c, &mut m1.data, &dir, "#26a69a", "#ef5350");
    vec![m0, m1]
}

fn compute_rsidiv(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let lw = num(o, "lineWidth", 1.0);
    let len = int(o, "length", 14);
    let closes: Vec<f64> = c.iter().map(|x| x.close).collect();
    let rsi = rsi_wilder(&closes, len);
    let data = points_from(c, &rsi);
    vec![mk_line(&strv(o, "color", "#b39ddb"), lw, data, None)]
}

fn markers_rsidiv(c: &[Candle], o: &Settings) -> Vec<Marker> {
    let res = divergence(c, int(o, "length", 14), int(o, "pivot", 5), int(o, "lookback", 200));
    let show_reg = boolv(o, "showRegular", true);
    let show_hid = boolv(o, "showHidden", true);
    let bull_c = strv(o, "bullColor", "#00d4aa");
    let bear_c = strv(o, "bearColor", "#ff5252");
    let mut mk = Vec::new();
    for s in res {
        if s.hidden && !show_hid {
            continue;
        }
        if !s.hidden && !show_reg {
            continue;
        }
        let bull = s.kind_bull;
        let (shape, position, text) = if bull {
            (if s.hidden { "circle" } else { "arrowUp" }, "belowBar", if s.hidden { "hidden bull" } else { "bull" })
        } else {
            (if s.hidden { "circle" } else { "arrowDown" }, "aboveBar", if s.hidden { "hidden bear" } else { "bear" })
        };
        mk.push(Marker {
            time: s.time,
            position: position.into(),
            color: if bull { bull_c.clone() } else { bear_c.clone() },
            shape: shape.into(),
            text: text.into(),
            size: 1.0,
        });
    }
    mk
}

fn compute_vlcore(c: &[Candle], o: &Settings) -> Vec<SeriesOut> {
    let up = strv(o, "upColor", "#00e676");
    let dn = strv(o, "downColor", "#ff5252");
    let flat = strv(o, "flatColor", "#6b6b88");
    let lw = num(o, "lineWidth", 2.0);
    let r = vlcore_engine(c, o);
    if r.val.is_empty() {
        return Vec::new();
    }
    let mut data = Vec::new();
    for i in 0..c.len().min(r.val.len()) {
        if let Some(v) = r.val[i] {
            if v.is_finite() {
                let col = if r.dir[i] == 1 { &up } else if r.dir[i] == -1 { &dn } else { &flat };
                data.push(Point { time: c[i].time, value: v, color: Some(col.clone()) });
            }
        }
    }
    if boolv(o, "straight", true) {
        data = straighten_line(&data, num(o, "straightTol", 0.08));
    }
    vec![mk_line(&up, lw, data, None)]
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------
pub fn registry() -> Vec<IndicatorEntry> {
    let mut v: Vec<IndicatorEntry> = Vec::new();

    v.push(e!(
        def("ema", "EMA", "Exponential Moving Average", "Overlay", IndType::Overlay, None,
            vec![num_in("length", "Length", 9.0, 1.0, 500.0, 1.0), source_in("close")],
            vec![color_st("color", "Color", "#2962ff"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_ema, "price"
    ));
    v.push(e!(
        def("ma", "MA", "Moving Average", "Overlay", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 500.0, 1.0), source_in("close")],
            vec![color_st("color", "Color", "#ff6d00"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_ma, "price"
    ));
    v.push(e!(
        def("smma", "Smoothed MA", "Smoothed Moving Average", "Overlay", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 500.0, 1.0), source_in("close")],
            vec![color_st("color", "Color", "#ffca28"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_smma, "price"
    ));
    v.push(e!(
        def("hma", "HMA", "Hull Moving Average", "Overlay", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 500.0, 1.0), source_in("close")],
            vec![color_st("color", "Color", "#00bcd4"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_hma, "price"
    ));
    v.push(e!(
        def("ao", "Awesome Oscillator", "Awesome Oscillator", "Momentum", IndType::Pane, None,
            vec![num_in("fast", "Fast length", 5.0, 1.0, 200.0, 1.0), num_in("slow", "Slow length", 34.0, 2.0, 500.0, 1.0)],
            vec![color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350")]),
        compute_ao, "number"
    ));
    v.push(e!(
        def("atr", "ATR", "Average True Range", "Volatility", IndType::Pane, None,
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#7e57c2"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_atr, "price"
    ));
    v.push(e!(
        def("adx", "ADX", "Average Directional Index", "Volatility", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("adxColor", "ADX color", "#e040fb"), color_st("diPlusColor", "+DI color", "#26a69a"),
                 color_st("diMinusColor", "-DI color", "#ef5350"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_adx, "percent"
    ));
    v.push(e!(
        def("bollingerB", "Bollinger Bands %B", "Bollinger Bands %B", "Volatility", IndType::Pane, Some("decimal"),
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 2.0, 0.1, 10.0, 0.1)],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_bollinger_b, "decimal"
    ));
    v.push(e!(
        def("bbpct", "BB%b", "Bollinger %B", "Trend", IndType::Pane, Some("decimal"),
            vec![num_in("length", "Length", 20.0, 2.0, 200.0, 1.0), num_in("mult", "Std.dev mult", 2.0, 0.1, 5.0, 0.1)],
            vec![color_st("color", "Color", "#ffb300"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_bbpct, "decimal"
    ));
    v.push(e!(
        def("macd", "MACD", "MACD", "Momentum", IndType::Pane, None,
            vec![num_in("fast", "Fast length", 12.0, 1.0, 200.0, 1.0), num_in("slow", "Slow length", 26.0, 2.0, 500.0, 1.0),
                 num_in("signal", "Signal length", 9.0, 1.0, 200.0, 1.0)],
            vec![color_st("macdColor", "MACD line", "#2962ff"), color_st("signalColor", "Signal line", "#ff6d00"),
                 color_st("histUpColor", "Histogram up", "#26a69a"), color_st("histDownColor", "Histogram down", "#ef5350")]),
        compute_macd, "number"
    ));
    v.push(e!(
        def("supertrend", "Supertrend", "Supertrend", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 10.0, 1.0, 200.0, 1.0), num_in("factor", "Factor", 3.0, 0.1, 10.0, 0.1)],
            vec![color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_supertrend, "price"
    ));
    v.push(e!(
        def("obv", "OBV", "On-Balance Volume", "Volume", IndType::Pane, None,
            vec![],
            vec![color_st("color", "Color", "#26a69a"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_obv, "number"
    ));
    v.push(e!(
        def("bb", "Bollinger Bands", "Bollinger Bands", "Volatility", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 2.0, 0.1, 10.0, 0.1)],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_bb, "price"
    ));
    v.push(e!(
        def("bbw", "BBW", "Bollinger Band Width", "Volume", IndType::Pane, Some("decimal"),
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 2.0, 0.1, 10.0, 0.1)],
            vec![color_st("color", "Color", "#ffb300"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_bbw, "number"
    ));
    v.push(e!(
        def("volosc", "Volume Oscillator", "Volume Oscillator", "Volume", IndType::Pane, Some("percent"),
            vec![num_in("fast", "Fast length", 5.0, 1.0, 200.0, 1.0), num_in("slow", "Slow length", 20.0, 2.0, 500.0, 1.0)],
            vec![color_st("color", "Color", "#26a69a"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_volosc, "percent"
    ));
    v.push(e!(
        def("ad", "Accumulation/Distribution", "Accumulation/Distribution Index", "Volume", IndType::Pane, None,
            vec![],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_ad, "number"
    ));
    v.push(e!(
        def("mfi", "MFI", "Money Flow Index", "Volume", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#ff6d00"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_mfi, "number"
    ));
    v.push(e!(
        def("pvt", "PVT", "Price Volume Trend", "Volume", IndType::Pane, None,
            vec![],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_pvt, "number"
    ));
    v.push(e!(
        def("dpo", "DPO", "Detrended Price Oscillator", "Momentum", IndType::Pane, None,
            vec![num_in("length", "Length", 21.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#7e57c2"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_dpo, "number"
    ));
    v.push(e!(
        def("ppo", "PPO", "Percentage Price Oscillator", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("fast", "Fast length", 12.0, 1.0, 200.0, 1.0), num_in("slow", "Slow length", 26.0, 2.0, 500.0, 1.0),
                 num_in("signal", "Signal length", 9.0, 1.0, 200.0, 1.0)],
            vec![color_st("ppoColor", "PPO line", "#2962ff"), color_st("signalColor", "Signal line", "#ff6d00"),
                 color_st("histUpColor", "Histogram up", "#26a69a"), color_st("histDownColor", "Histogram down", "#ef5350")]),
        compute_ppo, "percent"
    ));
    v.push(e!(
        def("williamsR", "Williams %R", "Williams Percent Range", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#e040fb"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_williams_r, "number"
    ));
    v.push(e!(
        def("rsi", "RSI", "Relative Strength Index", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0), source_in("close")],
            vec![color_st("color", "Color", "#7e57c2"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_rsi, "number"
    ));
    v.push(e!(
        def("uo", "UO", "Ultimate Oscillator", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("fast", "Fast length", 7.0, 1.0, 100.0, 1.0), num_in("mid", "Mid length", 14.0, 2.0, 200.0, 1.0),
                 num_in("slow", "Slow length", 28.0, 3.0, 500.0, 1.0)],
            vec![color_st("color", "Color", "#ff6d00"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_uo, "number"
    ));
    v.push(e!(
        def("vwap", "VWAP", "Volume Weighted Average Price", "Overlay", IndType::Overlay, None,
            vec![],
            vec![color_st("color", "Color", "#ffb300"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_vwap, "price"
    ));
    v.push(e!(
        def("pc", "Price Channel", "Price Channel", "Overlay", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 500.0, 1.0)],
            vec![color_st("color", "Color", "#26a69a"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_pc, "price"
    ));
    v.push(e!(
        def("cmf", "CMF", "Chaikin Money Flow", "Volume", IndType::Pane, None,
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_cmf, "number"
    ));
    v.push(e!(
        def("cci", "CCI", "Commodity Channel Index", "Momentum", IndType::Pane, None,
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#e040fb"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_cci, "number"
    ));
    v.push(e!(
        def("aroon", "Aroon", "Aroon Up/Down", "Trend", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("upColor", "Aroon Up", "#26a69a"), color_st("downColor", "Aroon Down", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_aroon, "number"
    ));
    v.push(e!(
        def("vortex", "Vortex", "Vortex Indicator (VI+ / VI-)", "Trend", IndType::Pane, None,
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0)],
            vec![color_st("plusColor", "VI+ color", "#26a69a"), color_st("minusColor", "VI- color", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_vortex, "number"
    ));
    v.push(e!(
        def("tsi", "TSI", "True Strength Index", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("long", "Long length", 25.0, 1.0, 200.0, 1.0), num_in("short", "Short length", 13.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#2962ff"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_tsi, "number"
    ));
    v.push(e!(
        def("donchian", "Donchian Channel", "Donchian Channel", "Trend", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 500.0, 1.0)],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_donchian, "price"
    ));
    v.push(e!(
        def("stochrsi", "Stoch RSI", "Stochastic RSI (K/D)", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("rsiLength", "RSI length", 14.0, 1.0, 200.0, 1.0), num_in("stochLength", "Stoch length", 14.0, 1.0, 200.0, 1.0),
                 num_in("k", "K smoothing", 3.0, 1.0, 50.0, 1.0), num_in("d", "D smoothing", 3.0, 1.0, 50.0, 1.0)],
            vec![color_st("kColor", "K color", "#2962ff"), color_st("dColor", "D color", "#ff6d00"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_stochrsi, "number"
    ));
    v.push(e!(
        def("elderforce", "Force Index", "Elder Force Index", "Momentum", IndType::Pane, None,
            vec![num_in("length", "Length", 13.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#7e57c2"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_elderforce, "number"
    ));
    v.push(e!(
        def("keltner", "Keltner Channels", "Keltner Channels (EMA + ATR)", "Volatility", IndType::Overlay, None,
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 2.0, 0.1, 10.0, 0.1)],
            vec![color_st("color", "Color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_keltner, "price"
    ));
    v.push(e!(
        def("chandelier", "Chandelier Exit", "Chandelier Exit (ATR trailing)", "Volatility", IndType::Overlay, None,
            vec![num_in("length", "Length", 22.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 3.0, 0.1, 10.0, 0.1)],
            vec![color_st("longColor", "Long color", "#26a69a"), color_st("shortColor", "Short color", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_chandelier, "price"
    ));
    v.push(e!(
        def("sqzmom", "Squeeze Momentum", "TTM Squeeze Momentum", "Volatility", IndType::Pane, None,
            vec![num_in("length", "Length", 20.0, 1.0, 200.0, 1.0), num_in("mult", "Mult", 2.0, 0.1, 10.0, 0.1)],
            vec![color_st("color", "Color", "#7e57c2"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_sqzmom, "number"
    ));
    v.push(e!(
        def("fisher", "Fisher Transform", "Fisher Transform (MESA)", "Momentum", IndType::Pane, None,
            vec![num_in("length", "Length", 9.0, 1.0, 200.0, 1.0)],
            vec![color_st("color", "Color", "#2962ff"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_fisher, "number"
    ));
    v.push(e!(
        def("ichimoku", "Ichimoku", "Ichimoku Cloud", "Trend", IndType::Overlay, None,
            vec![num_in("tenkan", "Tenkan", 9.0, 1.0, 200.0, 1.0), num_in("kijun", "Kijun", 26.0, 1.0, 200.0, 1.0),
                 num_in("senkou", "Senkou B", 52.0, 1.0, 500.0, 1.0)],
            vec![color_st("tenkanColor", "Tenkan color", "#2962ff"), color_st("kijunColor", "Kijun color", "#ef5350"),
                 color_st("spanAColor", "Span A color", "#26a69a"), color_st("spanBColor", "Span B color", "#ff6d00"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_ichimoku, "price"
    ));

    v.push(e!(
        def("pastruct", "Price Action Trend", "Price Action Structure (swing HH/HL + BOS/CHoCH, non-repaint)", "Trend", IndType::Overlay, None,
            vec![num_in("pivotLen", "Pivot length", 3.0, 1.0, 50.0, 1.0),
                 num_in("atrLen", "ATR length", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 0.25, 0.05, 5.0, 0.05),
                 enum_in("lineMode", "Line mode", "zigzag", &[("zigzag", "ZigZag"), ("trail", "Trailing")]),
                 check_in("markersOnly", "Markers only", false)],
            vec![bool_st("showMarkers", "Show markers", true),
                 color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_pastruct, markers_pastruct, "price"
    ));
    v.push(e!(
        def("pcr", "PCR EMA", "Put-Call Ratio + EMA overlay (pinned to a bottom band)", "Overlay", IndType::Overlay, Some("decimal"),
            vec![],
            vec![color_st("pcrColor", "PCR color", "#9e9e9e"), color_st("fastColor", "Fast color", "#00d4aa"),
                 color_st("slowColor", "Slow color", "#ff9800"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_pcr, "number"
    ));
    v.push(e!(
        def("pcrrail", "OI Rails", "OI Support/Resistance Rails + PCR (right-extended)", "Overlay", IndType::Overlay, None,
            vec![],
            vec![num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_pcrrail, "number"
    ));
    v.push(e!(
        def("iv", "IV", "Implied Volatility (IV) + Delta + Vega - live option greeks scaled onto the price axis as straight intersecting lines", "Volatility", IndType::Overlay, Some("decimal"),
            vec![],
            vec![color_st("color", "IV color", "#e040fb"), color_st("deltaColor", "Delta color", "#00bcd4"),
                 color_st("vegaColor", "Vega color", "#ff6d00"), num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_iv, "number"
    ));
    v.push(e!(
        def("projline", "Trend Projection", "Straight trend-line projection (pivot fit, extended into the future)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("fwd", "Project bars", 30.0, 1.0, 500.0, 1.0),
                 num_in("pivots", "Pivots used", 3.0, 2.0, 50.0, 1.0),
                 enum_in("mode", "Fit mode", "trendline", &[("trendline", "Trendline"), ("regression", "Regression")])],
            vec![color_st("histColor", "History color", "#7ee0ff"), color_st("projColor", "Projection color", "#b388ff"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_projline, markers_projline, "price"
    ));
    v.push(e!(
        def("wavefib", "Elliott Wave Trend", "Elliott wave structure: 1-2-3-4-5 / A-B-C labels + a single zigzag wave line", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![bool_st("showTrend", "Show trend", true), bool_st("showFib", "Show fib", false),
                 bool_st("showExt", "Show extensions", false), bool_st("showLabels", "Show labels", true),
                 color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 color_st("rangeColor", "Range color", "#9e9e9e"), color_st("fibColor", "Fib color", "#ffd54f"),
                 color_st("fibExtColor", "Fib ext color", "#ff8a65"), num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_wavefib, markers_wavefib, "price"
    ));
    v.push(e!(
        def_hidden("ewtrend", "Elliott Wave Trend (state)", "Elliott wave confirmed trend state (hidden; drives the AST filter)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 6.0, 0.1, 20.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![]),
        compute_ewtrend, "number"
    ));
    v.push(e!(
        def_hidden("patrend", "Price Action Trend (state)", "Price Action structure trend state (hidden; drives the AST filter)", "Trend", IndType::Overlay, None,
            vec![num_in("pivotLen", "Pivot length", 10.0, 1.0, 50.0, 1.0),
                 num_in("atrLen", "ATR length", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 0.25, 0.05, 5.0, 0.05)],
            vec![]),
        compute_patrend, "number"
    ));
    v.push(e!(
        def("keylevel", "Key Levels", "Key Levels (support/resistance zones scored by how often price respected them)", "Overlay", IndType::Overlay, None,
            vec![num_in("strength", "Pivot strength", 5.0, 1.0, 50.0, 1.0),
                 num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("minPct", "Min level %", 0.08, 0.0, 5.0, 0.01),
                 num_in("tolMult", "Cluster tol mult", 0.6, 0.1, 5.0, 0.1),
                 num_in("zones", "Zones", 5.0, 1.0, 10.0, 1.0)],
            vec![color_st("c1", "Level 1", "#2962ff"), color_st("c2", "Level 2", "#ff9800"),
                 color_st("c3", "Level 3", "#ef5350"), color_st("c4", "Level 4", "#26a69a"),
                 color_st("c5", "Level 5", "#ab47bc"), num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_keylevel, "price"
    ));
    v.push(e!(
        def("autotrend", "Auto Trendline", "Auto Trendline (best-respected support/resistance line through swing pivots)", "Overlay", IndType::Overlay, None,
            vec![num_in("strength", "Pivot strength", 5.0, 1.0, 50.0, 1.0),
                 num_in("look", "Look back", 60.0, 3.0, 500.0, 1.0),
                 num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("minPct", "Min line %", 0.05, 0.0, 5.0, 0.01),
                 num_in("tolMult", "Tol mult", 0.5, 0.05, 5.0, 0.05),
                 check_in("fullSpan", "Full span", true)],
            vec![color_st("upColor", "Support color", "#26a69a"), color_st("downColor", "Resistance color", "#ef5350"),
                 num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_autotrend, markers_autotrend, "price"
    ));
    v.push(e!(
        def("smiio", "SMI Ergodic Oscillator", "SMI Ergodic Oscillator", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("shortlen", "Short length", 13.0, 1.0, 100.0, 1.0),
                 num_in("longlen", "Long length", 25.0, 2.0, 200.0, 1.0),
                 num_in("siglen", "Signal length", 9.0, 1.0, 100.0, 1.0)],
            vec![color_st("color", "SMI color", "#7e57c2"), color_st("signalColor", "Signal color", "#ff6d00"),
                 color_st("histColor", "Histogram color", "#42a5f5"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_smiio, "number"
    ));
    v.push(e!(
        def("smf", "SMF", "Smart Money Flow (volume-pulse accumulation/distribution)", "Volume", IndType::Pane, Some("percent"),
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0),
                 num_in("signalLen", "Signal length", 9.0, 1.0, 100.0, 1.0),
                 num_in("volLen", "Volume length", 20.0, 1.0, 200.0, 1.0),
                 num_in("pulseCap", "Pulse cap", 3.0, 1.0, 10.0, 0.1)],
            vec![color_st("color", "SMF color", "#26c6da"), color_st("signalColor", "Signal color", "#ff6d00"),
                 color_st("histUpColor", "Histogram up", "#26a69a"), color_st("histDownColor", "Histogram down", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_smf, "number"
    ));
    v.push(e!(
        def("autosr", "Auto Support Resistance", "Auto Support Resistance (structure high/low lines)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min move %", 0.15, 0.0, 5.0, 0.05),
                 check_in("touchLine", "Touch line", true)],
            vec![color_st("resColor", "Resistance color", "#26a69a"), color_st("supColor", "Support color", "#ef5350"),
                 color_st("touchUpColor", "Touch up color", "#26a69a"), color_st("touchDownColor", "Touch down color", "#ef5350"),
                 num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_autosr, "price"
    ));
    v.push(e!(
        def("zzline", "ZigZag Trendline", "ATR ZigZag structure + straight pivot-fit intersection trendline (non-repaint)", "Trend", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("pivotLook", "Pivot look", 8.0, 2.0, 100.0, 1.0),
                 check_in("showZig", "Show zigzag", true), check_in("showLine", "Show trend line", true),
                 check_in("showBreaks", "Show breaks", true), check_in("fullSpan", "Full span", true)],
            vec![color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 color_st("trendColor", "Trend color", "#2962ff"), num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_zzline, markers_zzline, "price"
    ));
    v.push(e!(
        def("trendmaster", "Combo Master", "Combo Master: structure zigzag + straight intersection trendline + BOS/CHoCH + volume confirmation (non-repaint)", "Trend", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("trendLook", "Trend look", 12.0, 2.0, 100.0, 1.0),
                 check_in("lineOnly", "Line only", false), check_in("showLevel", "Show level", false),
                 check_in("showBreaks", "Show breaks", true), check_in("fullSpan", "Full span", true)],
            vec![color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 color_st("trendColor", "Trend color", "#2962ff"), color_st("levelColor", "Level color", "#ffb300"),
                 num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_trendmaster, markers_trendmaster, "price"
    ));
    v.push(e!(
        def("panemaster", "Pane Consensus Signal", "Pane Consensus Signal (all pane oscillators fused into one bullish/bearish straight intersection line)", "Trend", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("rsiLength", "RSI length", 14.0, 2.0, 100.0, 1.0),
                 num_in("bbLength", "BB length", 20.0, 2.0, 200.0, 1.0),
                 num_in("cciLength", "CCI length", 20.0, 2.0, 200.0, 1.0),
                 num_in("mfiLength", "MFI length", 14.0, 2.0, 200.0, 1.0),
                 num_in("stochLength", "Stoch length", 14.0, 2.0, 200.0, 1.0),
                 num_in("willrLength", "Williams length", 14.0, 2.0, 200.0, 1.0),
                 num_in("smooth", "Smooth", 3.0, 1.0, 50.0, 1.0),
                 num_in("fitLook", "Fit look", 60.0, 5.0, 300.0, 1.0),
                 num_in("minSeg", "Min segment", 3.0, 1.0, 50.0, 1.0),
                 num_in("bullTh", "Bull threshold", 3.0, 1.0, 8.0, 1.0),
                 num_in("bearTh", "Bear threshold", 3.0, 1.0, 8.0, 1.0),
                 check_in("useRSI", "Use RSI", true), check_in("useBB", "Use BB", true),
                 check_in("useStoch", "Use Stoch", true), check_in("useCCI", "Use CCI", true),
                 check_in("useWillR", "Use Williams", true), check_in("useMFI", "Use MFI", true),
                 check_in("useMACD", "Use MACD", true), check_in("useDiv", "Use divergence", true)],
            vec![color_st("upColor", "Up color", "#26a69a"), color_st("downColor", "Down color", "#ef5350"),
                 color_st("trendColor", "Trend color", "#2962ff"), color_st("exhaustColor", "Exhaustion color", "#ffb300"),
                 bool_st("showExhaustion", "Show exhaustion", true), num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_panemaster, markers_panemaster, "number"
    ));
    v.push(e!(
        def("pitchfork", "Pitchfork", "Andrews Pitchfork (median + parallel channel from 3 confirmed swings)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("minSpan", "Min span", 12.0, 3.0, 200.0, 1.0)],
            vec![color_st("medianColor", "Median color", "#2962ff"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_pitchfork, markers_pitchfork, "price"
    ));
    v.push(e!(
        def("fibfan", "Fibonacci Fan", "Fibonacci Fan (rays from a confirmed swing through fib ratios)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![color_st("fanColor", "Fan color", "#ab47bc"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_fibfan, markers_fibfan, "price"
    ));
    v.push(e!(
        def("gannfan", "Gann Fan", "Gann Fan (1x8 ... 8x1 rays scaled to the confirmed swing)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![color_st("fanColor", "Fan color", "#607d8b"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_gannfan, markers_gannfan, "price"
    ));
    v.push(e!(
        def_hidden("fibt", "Fibonacci Fan (state)", "Fibonacci Fan swing direction state (hidden; drives the AST filter)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![]),
        compute_fibt, "price"
    ));
    v.push(e!(
        def_hidden("gant", "Gann Fan (state)", "Gann Fan swing direction state (hidden; drives the AST filter)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05)],
            vec![]),
        compute_gant, "price"
    ));
    v.push(e!(
        def("srema", "S/R EMA Reversal", "Support Resistance EMA(1) Reversal (straight line)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("emaPeriod", "EMA period", 1.0, 1.0, 100.0, 1.0),
                 num_in("touchMult", "Touch mult", 0.5, 0.05, 5.0, 0.05),
                 num_in("touchWindow", "Touch window", 3.0, 1.0, 50.0, 1.0),
                 num_in("fwd", "Forward bars", 20.0, 1.0, 200.0, 1.0)],
            vec![color_st("bullColor", "Bull color", "#26a69a"), color_st("bearColor", "Bear color", "#ef5350"),
                 num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_srema, markers_srema, "price"
    ));
    v.push(e!(
        def_hidden("sremat", "S/R EMA Reversal (state)", "Support Resistance EMA(1) reversal direction state (hidden; drives the AST filter)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("emaPeriod", "EMA period", 1.0, 1.0, 100.0, 1.0),
                 num_in("touchMult", "Touch mult", 0.5, 0.05, 5.0, 0.05),
                 num_in("touchWindow", "Touch window", 3.0, 1.0, 50.0, 1.0),
                 num_in("fwd", "Forward bars", 20.0, 1.0, 200.0, 1.0)],
            vec![]),
        compute_sremat, "number"
    ));
    v.push(e!(
        def("supplydemand", "Supply Demand", "Supply Demand Structure (connected path + equal-length forecast)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrPeriod", "ATR period", 14.0, 2.0, 200.0, 1.0),
                 num_in("atrMult", "ATR mult", 2.0, 0.1, 10.0, 0.1),
                 num_in("minPct", "Min pivot %", 0.15, 0.0, 5.0, 0.05),
                 num_in("eqTol", "Equal tol %", 25.0, 0.0, 100.0, 1.0)],
            vec![color_st("structColor", "Structure color", "#b388ff"), color_st("liveColor", "Live color", "#7ee0ff"),
                 color_st("projColor", "Projection color", "#ffb74d"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_supplydemand, markers_supplydemand, "price"
    ));
    v.push(e!(
        def("vl", "VL", "Volume Line (volume-weighted trend, battery fade)", "Volume", IndType::Overlay, None,
            vec![num_in("length", "Length", 14.0, 1.0, 200.0, 1.0),
                 num_in("signalLen", "Signal length", 9.0, 1.0, 100.0, 1.0),
                 num_in("volLen", "Volume length", 20.0, 1.0, 200.0, 1.0),
                 check_in("straight", "Straighten", true)],
            vec![color_st("color", "Color", "#26c6da"), color_st("signalColor", "Signal color", "#ff6d00"),
                 num_st("straightTol", "Straighten tol", 0.08, 0.0, 1.0, 0.01), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_vl, markers_vl, "price"
    ));
    v.push(e!(
        def("rsidiv", "RSI Divergence", "RSI Divergence (Regular + Hidden)", "Momentum", IndType::Pane, Some("percent"),
            vec![num_in("length", "RSI length", 14.0, 2.0, 100.0, 1.0),
                 num_in("pivot", "Pivot strength", 5.0, 1.0, 50.0, 1.0),
                 num_in("lookback", "Look back", 200.0, 10.0, 1000.0, 1.0),
                 check_in("showRegular", "Show regular", true), check_in("showHidden", "Show hidden", true)],
            vec![color_st("color", "RSI color", "#b39ddb"), color_st("bullColor", "Bull color", "#00d4aa"),
                 color_st("bearColor", "Bear color", "#ff5252"), num_st("lineWidth", "Line width", 1.0, 1.0, 5.0, 1.0)]),
        compute_rsidiv, markers_rsidiv, "number"
    ));
    v.push(e!(
        def("vlcore", "Trend Core", "Trend Core (liquidity-grab / fake-breakout filtered, non-lagging)", "Overlay", IndType::Overlay, None,
            vec![num_in("atrLength", "ATR length", 14.0, 2.0, 200.0, 1.0),
                 num_in("length", "Length", 20.0, 1.0, 200.0, 1.0),
                 num_in("confirm", "Confirm", 3.0, 1.0, 50.0, 1.0),
                 num_in("gap", "Gap", 0.5, 0.0, 10.0, 0.1),
                 num_in("wickLen", "Wick length", 1.0, 0.0, 10.0, 0.1),
                 num_in("strongThr", "Strong threshold", 0.6, 0.0, 1.0, 0.05),
                 check_in("straightLine", "Straighten", true), check_in("useVolume", "Use volume", true)],
            vec![color_st("upColor", "Up color", "#00e676"), color_st("downColor", "Down color", "#ff5252"),
                 color_st("flatColor", "Flat color", "#6b6b88"), num_st("straightTol", "Straighten tol", 0.08, 0.0, 1.0, 0.01),
                 num_st("lineWidth", "Line width", 2.0, 1.0, 5.0, 1.0)]),
        compute_vlcore, markers_vlcore, "price"
    ));

    crate::patterns::registry_into(&mut v);

    v
}

pub fn find(id: &str) -> Option<IndicatorEntry> {
    registry().into_iter().find(|x| x.def.id == id)
}

pub fn catalog_json() -> serde_json::Value {
    let list: Vec<IndicatorDef> = registry().into_iter().map(|x| x.def).collect();
    serde_json::to_value(list).unwrap_or(serde_json::Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synth(n: usize) -> Vec<Candle> {
        let mut out = Vec::with_capacity(n);
        let mut price = 100.0_f64;
        let mut t = 1_700_000_000_i64;
        for i in 0..n {
            let wave = ((i as f64) * 0.13).sin() * 2.5 + ((i as f64) * 0.041).cos() * 0.9;
            let open = price;
            let close = (price + wave).max(1.0);
            let high = open.max(close) + (i % 5) as f64 * 0.3 + 0.2;
            let low = open.min(close) - (i % 3) as f64 * 0.25 - 0.15;
            out.push(Candle { time: t, open, high, low, close, volume: 1000.0 + (i % 17) as f64 * 87.0 });
            price = close;
            t += 300;
        }
        out
    }

    fn defaults(entry: &IndicatorEntry) -> Settings {
        let mut s = Settings::new();
        for inp in &entry.def.inputs {
            s.insert(inp.key.clone(), inp.def.clone());
        }
        for st in &entry.def.style {
            s.insert(st.key.clone(), st.def.clone());
        }
        s
    }

    #[test]
    fn straight_line_arrow_markers_flip_with_direction() {
        // Arrow markers are printed exactly where a line's slope changes sign.
        let mut line = SeriesOut::line("#26a69a", 1.0);
        line.data = vec![
            Point { time: 1, value: 1.0, color: None },
            Point { time: 2, value: 2.0, color: None },
            Point { time: 3, value: 3.0, color: None },
            Point { time: 4, value: 2.5, color: None },
            Point { time: 5, value: 2.0, color: None },
            Point { time: 6, value: 2.5, color: None },
        ];
        let mk = slope_markers(&[line], 0, "#00ff00", "#ff0000");
        let got: Vec<(&str, i64)> = mk.iter().map(|m| (m.shape.as_str(), m.time)).collect();
        assert_eq!(got, vec![("arrowUp", 2), ("arrowDown", 4), ("arrowUp", 6)]);
    }

    #[test]
    fn straight_line_indicators_expose_arrow_markers() {
        let reg = registry();
        for id in ["vlcore", "vl", "srema", "autotrend", "pitchfork", "projline", "fibfan", "gannfan", "supplydemand"] {
            let e = reg.iter().find(|e| e.def.id == id).unwrap();
            assert!(e.markers.is_some(), "{id} must expose up/down arrow markers");
        }
    }

    #[test]
    fn straight_line_markers_are_emitted_for_real_series() {
        let candles = synth(400);
        let reg = registry();
        for id in ["zzline", "trendmaster", "panemaster", "vlcore", "vl", "srema", "autotrend", "pitchfork", "projline", "fibfan", "gannfan", "supplydemand"] {
            let e = reg.iter().find(|e| e.def.id == id).unwrap();
            let mk = (e.markers.unwrap())(&candles, &defaults(e));
            assert!(!mk.is_empty(), "{id} must emit trend-start arrows on a real oscillating series, got 0");
        }
    }

    fn trend_then_drop() -> Vec<Candle> {
        let mut c = synth(200);
        let mut price = c.last().unwrap().close;
        let mut t = c.last().unwrap().time;
        for _ in 0..40 {
            let open = price;
            let close = price - 0.9;
            t += 300;
            c.push(Candle {
                time: t,
                open,
                high: open + 0.2,
                low: close - 0.2,
                close,
                volume: 1000.0,
            });
            price = close;
        }
        c
    }

    #[test]
    fn live_trend_dir_flips_same_bar_as_the_turn() {
        let c = trend_then_drop();
        let dir = live_trend_dir(&c, 9);
        assert_eq!(dir[c.len() - 1], -1, "trend must be bearish right after the down leg");
        assert_eq!(dir[210], -1, "trend must be bearish a few bars into the drop");
    }

    #[test]
    fn straight_line_family_colors_follow_live_trend() {
        let c = trend_then_drop();
        let last_time = c.last().unwrap().time;
        let reg = registry();
        for id in [
            "zzline", "trendmaster", "panemaster", "srema", "autotrend", "pitchfork",
            "projline", "fibfan", "gannfan", "supplydemand", "vl",
        ] {
            let e = reg.iter().find(|e| e.def.id == id).unwrap();
            let out = (e.compute)(&c, &defaults(e));
            let last = out
                .iter()
                .find_map(|s| {
                    s.data
                        .last()
                        .filter(|p| p.time == last_time)
                        .and_then(|p| p.color.clone())
                })
                .unwrap_or_else(|| panic!("{id} produced no trend-colored point at the last bar"));
            assert!(
                last.contains("ef5350"),
                "{id} last point must be bearish-colored after the down leg, got {last}"
            );
        }
    }

    #[test]
    fn registry_matches_old_app_catalog() {
        let reg = registry();
        // 65 old-app indicators + 27 candlestick patterns (crate::patterns).
        assert_eq!(reg.len(), 92);
        let find = |id: &str| reg.iter().find(|e| e.def.id == id).unwrap();
        for id in ["pcr", "pcrrail", "iv", "panemaster", "ewtrend", "patrend", "sremat"] {
            assert_eq!(find(id).def.kind, IndType::Overlay, "{id} must be an overlay");
        }
        for id in ["ewtrend", "fibt", "gant", "patrend", "sremat"] {
            assert!(find(id).def.hidden, "{id} must be hidden");
        }
    }

    #[test]
    fn all_indicators_compute_without_panic() {
        for n in [5usize, 60, 400] {
            let candles = synth(n);
            for entry in registry() {
                let settings = defaults(&entry);
                let out = (entry.compute)(&candles, &settings);
                for series in &out {
                    assert!(
                        series.data.len() <= n + 600,
                        "{} produced too many points at n={n}",
                        entry.def.id
                    );
                }
                if let Some(mk) = entry.markers {
                    let _ = mk(&candles, &settings);
                }
            }
        }
    }
}
