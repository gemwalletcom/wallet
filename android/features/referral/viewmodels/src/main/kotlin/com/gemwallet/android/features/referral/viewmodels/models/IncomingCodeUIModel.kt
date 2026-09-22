package com.gemwallet.android.features.referral.viewmodels.models

import uniffi.gemstone.GemIncomingCode

data class IncomingCodeUIModel(val confirm: String? = null, val activate: String? = null)

fun GemIncomingCode?.uiModel() = IncomingCodeUIModel(
    confirm = (this as? GemIncomingCode.Confirm)?.code,
    activate = (this as? GemIncomingCode.Activate)?.code,
)
