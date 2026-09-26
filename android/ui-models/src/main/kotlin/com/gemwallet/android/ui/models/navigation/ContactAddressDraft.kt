package com.gemwallet.android.ui.models.navigation

import com.wallet.core.primitives.Chain

data class ContactAddressDraft(val chain: Chain, val address: String, val memo: String?)
