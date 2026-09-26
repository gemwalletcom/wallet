package com.gemwallet.android.features.assets.presents.select

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import com.gemwallet.android.features.assets.viewmodels.select.AssetSelectViewModel
import uniffi.gemstone.GemSelectAssetType

@Composable
fun assetSelectViewModel(selectType: GemSelectAssetType): AssetSelectViewModel = hiltViewModel<AssetSelectViewModel, AssetSelectViewModel.Factory>(key = selectType.toString()) { it.create(selectType) }
