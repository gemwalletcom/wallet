package com.gemwallet.android.features.perpetuals.viewmodels.models

import uniffi.gemstone.GemFormattedNumber

data class PerpetualTradeRowUIModel(val balance: GemFormattedNumber, val hideBalance: Boolean = false)
