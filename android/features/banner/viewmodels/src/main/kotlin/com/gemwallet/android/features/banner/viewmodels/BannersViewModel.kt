package com.gemwallet.android.features.banner.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.banner.coordinators.CancelBanner
import com.gemwallet.android.application.banner.coordinators.GetActiveBanners
import com.gemwallet.android.ext.getReserveBalance
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Banner
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.math.BigInteger
import javax.inject.Inject

@HiltViewModel
class BannersViewModel @Inject constructor(
    private val getActiveBanners: GetActiveBanners,
    private val cancelBanner: CancelBanner,
) : ViewModel() {

    val banners = MutableStateFlow<List<Banner>>(emptyList())

    fun init(asset: Asset?, isGlobal: Boolean) {
        viewModelScope.launch(Dispatchers.IO) {
            val banners = getActiveBanners(asset, isGlobal)
            this@BannersViewModel.banners.update { banners }
        }
    }

    fun getActivationFee(asset: Asset?): String {
        asset ?: return ""
        val value = asset.id.chain.getReserveBalance()
        if (value == BigInteger.ZERO) return ""
        return asset.format(Crypto(value))
    }

    fun onCancel(banner: Banner) = viewModelScope.launch {
        cancelBanner(banner)
        init(banner.asset, banner.wallet == null)
    }
}
