package com.gemwallet.android.features.main.viewmodels

import androidx.lifecycle.ViewModel
import com.gemwallet.android.application.asset_select.coordinators.GetSelectAssetsInfo
import com.gemwallet.android.ext.decodePayment
import com.gemwallet.android.ext.request
import com.gemwallet.android.model.PaymentDestination
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.first
import javax.inject.Inject

@HiltViewModel
class PaymentScanViewModel @Inject constructor(
    private val getSelectAssetsInfo: GetSelectAssetsInfo,
) : ViewModel() {

    suspend fun onScan(scanned: String): PaymentDestination {
        val request = decodePayment(scanned)?.request ?: return PaymentDestination.Unsupported

        return PaymentDestination.from(request, getSelectAssetsInfo().first())
    }
}
