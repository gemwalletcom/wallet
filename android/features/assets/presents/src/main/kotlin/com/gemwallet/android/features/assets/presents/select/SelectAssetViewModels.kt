package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.assets.viewmodels.select.SelectAssetViewModel
import uniffi.gemstone.GemSelectAssetType

@Composable
fun selectAssetViewModel(selectType: GemSelectAssetType): SelectAssetViewModel = hiltViewModel<SelectAssetViewModel, SelectAssetViewModel.Factory>(key = selectType.toString()) { it.create(selectType) }
