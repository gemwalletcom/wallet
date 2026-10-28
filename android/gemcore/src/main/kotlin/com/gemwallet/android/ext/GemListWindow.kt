package com.gemwallet.android.ext

import uniffi.gemstone.GemListWindow

val GemListWindow.Companion.transactions: GemListWindow
    get() = GemListWindow(limit = GemConstants.listPageSize, maximum = GemConstants.transactionsListLimit)
