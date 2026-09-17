package com.gemwallet.android.features.asset_select.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.serializer.unpackRoutePayload
import com.gemwallet.android.ui.models.navigation.RouteArgument

internal fun SavedStateHandle.paymentAssetIds(): List<String> =
    get<String>(RouteArgument.AssetIds.key)?.let { unpackRoutePayload<List<String>>(it) }.orEmpty()
