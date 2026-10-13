package com.gemwallet.android.features.rewards.viewmodels.models

sealed interface RewardsSheetType {
    data object CreateCode : RewardsSheetType
    data class ActivateCode(val code: String) : RewardsSheetType
}
