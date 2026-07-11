package com.gemwallet.android.ui.components

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Resource

@StringRes
fun Resource.titleRes(): Int = when (this) {
    Resource.Bandwidth -> R.string.stake_resource_bandwidth
    Resource.Energy -> R.string.stake_resource_energy
}
