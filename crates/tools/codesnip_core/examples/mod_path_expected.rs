mod a {
    //! a.rs
}
mod b {
    //! b/mod.rs
}
#[path = "c_path.rs"]
mod c {
    //! c_path.rs
}
mod d {
    mod da {
        //! d/da.rs
    }
    mod db {
        //! d/db/mod.rs
    }
    #[path = "dc_path.rs"]
    mod dc {
        //! d/dc_path.rs
    }
}
#[path = "e_path"]
mod e {
    mod ea {
        //! e_path/ea.rs
    }
    mod eb {
        //! e_path/eb/mod.rs
    }
    #[path = "ec_path.rs"]
    mod ec {
        //! e_path/ec_path.rs
    }
}
mod x {
    //! x.rs
    mod xa {
        //! x/xa.rs
    }
    mod xb {
        //! x/xb.rs
    }
    #[path = "xc_path.rs"]
    mod xc {
        //! xc_path.rs
    }
    mod xd {
        mod xda {
            //! x/xd/xda.rs
        }
        mod xdb {
            //! x/xd/xdb/mod.rs
        }
        #[path = "xdc_path.rs"]
        mod xdc {
            //! x/xd/xdc_path.rs
        }
    }
    #[path = "xe_path"]
    mod xe {
        mod xea {
            //! xe_path/xea.rs
        }
        mod xeb {
            //! xe_path/xeb/mod.rs
        }
        #[path = "xec_path.rs"]
        mod xec {
            //! xe_path/xec_path.rs
        }
    }
}
mod y {
    //! y/mod.rs
    mod ya {
        //! y/ya.rs
    }
    mod yb {
        //! y/yb/mod.rs
    }
    #[path = "yc_path.rs"]
    mod yc {
        //! y/yc_path.rs
    }
    mod yd {
        mod yda {
            //! y/yd/yda.rs
        }
        mod ydb {
            //! y/yd/ydb/mod.rs
        }
        #[path = "ydc_path.rs"]
        mod ydc {
            //! y/yd/ydc_path.rs
        }
    }
    #[path = "ye_path"]
    mod ye {
        mod yea {
            //! y/ye_path/yea.rs
        }
        mod yeb {
            //! y/ye_path/yeb/mod.rs
        }
        #[path = "yec_path.rs"]
        mod yec {
            //! y/ye_path/yec_path.rs
        }
    }
}
#[path = "z_path.rs"]
mod z {
    //! z_path.rs
    mod za {
        //! za.rs
    }
    mod zb {
        //! zb/mod.rs
    }
    #[path = "zc_path.rs"]
    mod zc {
        //! zc_path.rs
    }
    mod zd {
        mod zda {
            //! zd/zda.rs
        }
        mod zdb {
            //! zd/zdb/mod.rs
        }
        #[path = "zdc_path.rs"]
        mod zdc {
            //! zd/zdc_path.rs
        }
    }
    #[path = "ze_path"]
    mod ze {
        mod zea {
            //! ze_path/zea.rs
        }
        mod zeb {
            //! ze_path/zeb/mod.rs
        }
        #[path = "zec_path.rs"]
        mod zec {
            //! ze_path/zec_path.rs
        }
    }
}
