package com.gemwallet.android.ext

import com.wallet.core.primitives.Transaction

val Transaction.hash: String
    get() = id.hash
