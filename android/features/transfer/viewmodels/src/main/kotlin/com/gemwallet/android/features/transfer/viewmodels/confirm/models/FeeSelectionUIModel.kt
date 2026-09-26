package com.gemwallet.android.features.transfer.viewmodels.confirm.models

import com.wallet.core.primitives.FeePriority
import java.math.BigInteger

data class FeeSelectionUIModel(val selectedPriority: FeePriority?, val customRate: BigInteger?)
