package com.gemwallet.android.features.banner.viewmodels

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toGemKey
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemBannerAction
import uniffi.gemstone.GemBannerContent
import uniffi.gemstone.GemBannerServiceInterface
import javax.inject.Inject

data class BannerUIModel(
    val banner: Banner,
    val content: GemBannerContent,
)

private data class BannerScene(val asset: Asset?, val isGlobal: Boolean)

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class BannersViewModel @Inject constructor(
    getActiveBanners: GetActiveBanners,
    private val service: GemBannerServiceInterface,
) : ViewModel() {

    private val scene = MutableStateFlow<BannerScene?>(null)

    val banners = scene.filterNotNull()
        .flatMapLatest { getActiveBanners(it.asset, it.isGlobal) }
        .map { items -> items.map { BannerUIModel(it, service.bannerContent(it.event.toJson(), it.asset?.toGem())) } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun init(asset: Asset?, isGlobal: Boolean) {
        scene.value = BannerScene(asset, isGlobal)
    }

    fun onSelect(banner: Banner) = apply(banner, GemBannerAction.Event(banner.event.toJson()))

    fun onCancel(banner: Banner) = apply(banner, GemBannerAction.Close)

    private fun apply(banner: Banner, action: GemBannerAction) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.applyAction(banner.toGemKey(), action) }
            .onFailure { Log.e(TAG, "banner ${banner.event} action failed", it) }
    }

    private companion object {
        const val TAG = "Banners"
    }
}
