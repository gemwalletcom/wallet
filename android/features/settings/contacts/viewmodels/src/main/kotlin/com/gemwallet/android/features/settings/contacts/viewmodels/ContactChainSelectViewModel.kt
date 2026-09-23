package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.lifecycle.ViewModel
import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemChainServiceInterface
import javax.inject.Inject

@HiltViewModel
class ContactChainSelectViewModel @Inject constructor(private val chainService: GemChainServiceInterface) : ViewModel() {
    fun chains(query: String): List<Chain> = chainService.getChains(query).map { it.requireChain() }
}
