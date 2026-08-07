package com.gemwallet.android.features.stake.models

data class StakeActionItem(
    val action: StakeAction,
    val enabled: Boolean,
    val frozenRequired: Boolean,
)
