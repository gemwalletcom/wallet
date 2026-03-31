package com.gemwallet.android.ui.models

import com.wallet.core.primitives.Asset

val Asset.subtitleSymbol: String?
    get() = symbol.takeIf { it != name }
