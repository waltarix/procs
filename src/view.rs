use crate::column::Column;
use crate::columns::*;
use crate::config::*;
use crate::process::collect_proc;
use crate::style::{apply_color, apply_style, color_to_column_style};
use crate::term_info::TermInfo;
use crate::util::{classify, find_column_kind, find_exact, find_partial, truncate, KeywordClass};
use crate::Opt;
use anyhow::Error;
#[cfg(not(target_os = "windows"))]
use pager::Pager;
use std::collections::HashMap;
use std::time::Duration;

pub struct SortInfo {
    pub idx: usize,
    pub order: ConfigSortOrder,
}

pub struct View {
    pub columns: Vec<ColumnInfo>,
    pub term_info: TermInfo,
    pub sort_info: SortInfo,
    pub visible_pids: Vec<i32>,
    pub auxiliary_pids: Vec<i32>,
    pub ppids: HashMap<i32, i32>,
}

impl View {
    pub fn new(opt: &Opt, config: &Config, clear_by_line: bool) -> Self {
        let mut slot_idx = 0;
        let mut columns = Vec::new();
        if opt.tree {
            let kind = ConfigColumnKind::Tree;
            let column = gen_column(
                &kind,
                None,
                &config.docker.path,
                &config.display.separator,
                config.display.abbr_sid,
                &config.display.tree_symbols,
            );
            if column.available() {
                columns.push(ColumnInfo {
                    column,
                    kind,
                    style: color_to_column_style(&config.style.tree),
                    nonnumeric_search: false,
                    numeric_search: false,
                    align: ConfigColumnAlign::Left,
                    max_width: None,
                    min_width: None,
                });
            }
        }
        for c in &config.columns {
            let kind = match &c.kind {
                ConfigColumnKind::Slot => {
                    let kind = if let Some(insert) = opt.insert.get(slot_idx) {
                        find_column_kind(insert)
                    } else {
                        None
                    };
                    slot_idx += 1;
                    kind
                }
                x => Some(x.clone()),
            };
            if let Some(kind) = kind {
                let column = gen_column(
                    &kind,
                    c.header.clone(),
                    &config.docker.path,
                    &config.display.separator,
                    config.display.abbr_sid,
                    &config.display.tree_symbols,
                );
                if column.available() {
                    columns.push(ColumnInfo {
                        column,
                        kind,
                        style: c.style.clone(),
                        nonnumeric_search: c.nonnumeric_search,
                        numeric_search: c.numeric_search,
                        align: c.align.clone(),
                        max_width: c.max_width,
                        min_width: c.min_width,
                    });
                }
            }
        }

        let proc = collect_proc(Duration::from_millis(opt.interval));
        for c in columns.iter_mut() {
            for p in &proc {
                c.column.add(&p);
            }
        }

        let mut ppids = HashMap::new();
        for p in &proc {
            ppids.insert(p.pid, p.ppid);
        }

        let term_info = TermInfo::new(clear_by_line);
        let sort_info = View::get_sort_info(opt, config, &columns);

        View {
            columns,
            term_info,
            sort_info,
            visible_pids: vec![],
            auxiliary_pids: vec![],
            ppids,
        }
    }

    pub fn filter(&mut self, opt: &Opt, config: &Config) {
        let mut cols_nonnumeric = Vec::new();
        let mut cols_numeric = Vec::new();
        for c in &self.columns {
            if c.nonnumeric_search {
                cols_nonnumeric.push(c.column.as_ref());
            }
            if c.numeric_search {
                cols_numeric.push(c.column.as_ref());
            }
        }

        let mut keyword_nonnumeric = Vec::new();
        let mut keyword_numeric = Vec::new();

        for k in &opt.keyword {
            match classify(k) {
                KeywordClass::Numeric => keyword_numeric.push(k),
                KeywordClass::NonNumeric => keyword_nonnumeric.push(k),
            }
        }

        let pids = self.columns[self.sort_info.idx]
            .column
            .sorted_pid(&self.sort_info.order);

        let self_pid = std::process::id() as i32;

        let logic = if opt.and {
            ConfigSearchLogic::And
        } else if opt.or {
            ConfigSearchLogic::Or
        } else if opt.nand {
            ConfigSearchLogic::Nand
        } else if opt.nor {
            ConfigSearchLogic::Nor
        } else {
            config.search.logic.clone()
        };

        let mut candidate_pids = Vec::new();
        for pid in &pids {
            let candidate = if !config.display.show_self && *pid == self_pid {
                false
            } else if opt.keyword.is_empty() {
                true
            } else {
                View::search(
                    *pid,
                    &keyword_numeric,
                    &keyword_nonnumeric,
                    cols_numeric.as_slice(),
                    cols_nonnumeric.as_slice(),
                    &config,
                    &logic,
                )
            };

            if candidate {
                candidate_pids.push(*pid);
            }
        }

        let mut auxiliary_pids = Vec::new();
        if opt.tree {
            let mut additional_pids = Vec::new();
            for pid in &candidate_pids {
                additional_pids.append(&mut self.get_ppids(*pid));
            }
            let mut additional_pids: Vec<_> = additional_pids
                .iter()
                .filter(|x| !candidate_pids.contains(x))
                .map(|x| *x)
                .collect();
            candidate_pids.append(&mut additional_pids.clone());
            auxiliary_pids.append(&mut additional_pids);
        }

        let mut visible_pids = Vec::new();
        for pid in &pids {
            if candidate_pids.contains(pid) {
                visible_pids.push(*pid);
            }

            if opt.watch_mode && visible_pids.len() >= self.term_info.height - 5 {
                break;
            }
        }

        self.visible_pids = visible_pids;
        self.auxiliary_pids = auxiliary_pids;
    }

    fn get_ppids(&self, pid: i32) -> Vec<i32> {
        let mut ret = vec![];
        if let Some(x) = self.ppids.get(&pid) {
            ret.push(*x);
            ret.append(&mut self.get_ppids(*x));
            ret
        } else {
            ret
        }
    }

    pub fn adjust(&mut self, config: &Config, min_widths: &HashMap<usize, usize>) {
        for (i, ref mut c) in self.columns.iter_mut().enumerate() {
            let order = if i == self.sort_info.idx {
                Some(self.sort_info.order.clone())
            } else {
                None
            };
            c.column.apply_visible(&self.visible_pids);
            let min_width = min_widths.get(&i).map(|x| Some(*x)).unwrap_or(c.min_width);
            c.column.reset_width(order, &config, c.max_width, min_width);
            for pid in &self.visible_pids {
                c.column.update_width(*pid, c.max_width);
            }
        }
    }

    pub fn display(&mut self, opt: &Opt, config: &Config) -> Result<(), Error> {
        let use_terminal = console::user_attended();

        // +3 means header/unit line and next prompt
        let pager_threshold = self.visible_pids.len() + 3;

        let use_pager = if cfg!(target_os = "windows") {
            false
        } else {
            match (opt.watch_mode, opt.pager.as_ref(), &config.pager.mode) {
                (true, _, _) => false,
                (false, Some(x), _) if x == "auto" => self.term_info.height < pager_threshold,
                (false, Some(x), _) if x == "always" => true,
                (false, Some(x), _) if x == "disable" => false,
                (false, None, ConfigPagerMode::Auto) => self.term_info.height < pager_threshold,
                (false, None, ConfigPagerMode::Always) => true,
                (false, None, ConfigPagerMode::Disable) => false,
                _ => false,
            }
        };

        let mut truncate = use_terminal && use_pager && config.display.cut_to_pager;
        truncate |= use_terminal && !use_pager && config.display.cut_to_terminal;
        truncate |= !use_terminal && config.display.cut_to_pipe;

        if !truncate {
            self.term_info.width = std::usize::MAX;
        }

        if use_pager {
            View::pager(&config);
        }

        match (opt.color.as_ref(), &config.display.color_mode) {
            (Some(x), _) if x == "auto" => {
                if use_pager && use_terminal {
                    console::set_colors_enabled(true);
                }
            }
            (Some(x), _) if x == "always" => console::set_colors_enabled(true),
            (Some(x), _) if x == "disable" => console::set_colors_enabled(false),
            (None, ConfigColorMode::Auto) => {
                if use_pager && use_terminal {
                    console::set_colors_enabled(true);
                }
            }
            (None, ConfigColorMode::Always) => console::set_colors_enabled(true),
            (None, ConfigColorMode::Disable) => console::set_colors_enabled(false),
            _ => (),
        }

        // Ignore display_* error
        //   `Broken pipe` may occur at pager mode. It can be ignored safely.
        let _ = self.display_header(config);
        let _ = self.display_unit(&config);

        for pid in &self.visible_pids {
            let auxiliary = self.auxiliary_pids.contains(pid);
            let _ = self.display_content(&config, *pid, auxiliary);
        }

        Ok(())
    }

    fn display_header(&self, config: &Config) -> Result<(), Error> {
        let mut row = String::from("");
        for (i, c) in self.columns.iter().enumerate() {
            let order = if i == self.sort_info.idx {
                Some(self.sort_info.order.clone())
            } else {
                None
            };
            row = format!(
                "{} {}",
                row,
                apply_color(
                    c.column.display_header(&c.align, order, config),
                    &config.style.header,
                    false
                )
            );
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_unit(&self, config: &Config) -> Result<(), Error> {
        let mut row = String::from("");
        for c in &self.columns {
            row = format!(
                "{} {}",
                row,
                apply_color(c.column.display_unit(&c.align), &config.style.unit, false)
            );
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn display_content(&self, config: &Config, pid: i32, auxiliary: bool) -> Result<(), Error> {
        let mut row = String::from("");
        for c in &self.columns {
            row = format!(
                "{} {}",
                row,
                apply_style(
                    c.column.display_content(pid, &c.align).unwrap(),
                    &c.style,
                    &config.style,
                    auxiliary
                )
            );
        }
        row = row.trim_end().to_string();
        row = truncate(&row, self.term_info.width).to_string();
        self.term_info.write_line(&row)?;
        Ok(())
    }

    fn get_sort_info(opt: &Opt, config: &Config, cols: &[ColumnInfo]) -> SortInfo {
        let (mut sort_idx, sort_order) = match (&opt.sorta, &opt.sortd) {
            (Some(sort), _) | (_, Some(sort)) => {
                let mut idx = config.sort.column;
                let mut order = config.sort.order.clone();
                for (i, c) in cols.iter().enumerate() {
                    let (kind, _) = KIND_LIST[&c.kind];
                    if kind.to_lowercase().find(&sort.to_lowercase()).is_some() {
                        idx = i;
                        order = if opt.sorta.is_some() {
                            ConfigSortOrder::Ascending
                        } else {
                            ConfigSortOrder::Descending
                        };
                        break;
                    }
                }
                (idx, order)
            }
            _ => (config.sort.column, config.sort.order.clone()),
        };

        if opt.tree {
            sort_idx = 0;
        }

        SortInfo {
            idx: sort_idx,
            order: sort_order,
        }
    }

    fn search<T: AsRef<str>>(
        pid: i32,
        keyword_numeric: &[T],
        keyword_nonnumeric: &[T],
        cols_numeric: &[&dyn Column],
        cols_nonnumeric: &[&dyn Column],
        config: &Config,
        logic: &ConfigSearchLogic,
    ) -> bool {
        let ret_nonnumeric = match config.search.nonnumeric_search {
            ConfigSearchKind::Partial => {
                find_partial(cols_nonnumeric, pid, keyword_nonnumeric, logic, config.search.smart_case)
            }
            ConfigSearchKind::Exact => find_exact(cols_nonnumeric, pid, keyword_nonnumeric, logic),
        };
        let ret_numeric = match config.search.numeric_search {
            ConfigSearchKind::Partial => find_partial(cols_numeric, pid, keyword_numeric, logic, false),
            ConfigSearchKind::Exact => find_exact(cols_numeric, pid, keyword_numeric, logic),
        };
        match logic {
            ConfigSearchLogic::And => ret_nonnumeric & ret_numeric,
            ConfigSearchLogic::Or => ret_nonnumeric | ret_numeric,
            ConfigSearchLogic::Nand => !(ret_nonnumeric & ret_numeric),
            ConfigSearchLogic::Nor => !(ret_nonnumeric | ret_numeric),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn pager(config: &Config) {
        if let Some(ref pager) = config.pager.command {
            Pager::with_pager(&pager).setup();
        } else if which::which("less").is_ok() {
            Pager::with_pager("less -SR").setup();
        } else {
            Pager::with_pager("more -f").setup();
        }
    }

    #[cfg(target_os = "windows")]
    fn pager(_config: &Config) {}

    #[cfg_attr(tarpaulin, skip)]
    pub fn inc_sort_column(&mut self) -> usize {
        let current = self.sort_info.idx;
        let max_idx = self.columns.len();

        for i in 1..max_idx {
            let idx = (current + i) % max_idx;
            if self.columns[idx].column.sortable() {
                return idx;
            }
        }
        current
    }

    #[cfg_attr(tarpaulin, skip)]
    pub fn dec_sort_column(&mut self) -> usize {
        let current = self.sort_info.idx;
        let max_idx = self.columns.len();

        for i in 1..max_idx {
            let idx = (current + max_idx - i) % max_idx;
            if self.columns[idx].column.sortable() {
                return idx;
            }
        }
        current
    }
}
