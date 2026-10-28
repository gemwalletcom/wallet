package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemSelectAssetType

@Composable
fun selectAssetViewModel(selectType: GemSelectAssetType, chains: List<Chain> = emptyList()): SelectAssetViewModel =
    hiltViewModel<SelectAssetViewModel, SelectAssetViewModel.Factory>(key = selectType.toString()) { it.create(selectType, chains) }
