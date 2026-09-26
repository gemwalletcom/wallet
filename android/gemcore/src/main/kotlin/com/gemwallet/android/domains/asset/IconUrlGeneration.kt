package com.gemwallet.android.domains.asset

import com.gemwallet.android.ext.chainConfig
import com.gemwallet.android.ext.toChain
import com.wallet.core.primitives.Chain

fun Chain.iconChain(): Chain = chainConfig().iconChain.toChain()
