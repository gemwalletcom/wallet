package com.gemwallet.android.features.recipient.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.recipient.coordinators.GetWallets
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.cases.contacts.ContactRecipient
import com.gemwallet.android.cases.contacts.GetContacts
import com.gemwallet.android.cases.name.ResolveName
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.mutableStateIn
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.optionalNftAssetId
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NameRecord
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecipientViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getWallets: GetWallets,
    private val getContacts: GetContacts,
    private val getAssetInfo: GetAssetInfo,
    private val getAssetNft: GetAssetNft,
    private val validateAddressOperator: ValidateAddressOperator,
    private val resolveName: ResolveName,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    private val _address = MutableStateFlow("")
    val address = _address.asStateFlow()

    private val _memo = MutableStateFlow("")
    val memo = _memo.asStateFlow()

    private val nameRecord = MutableStateFlow<NameRecord?>(null)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)
    private val nftAssetId = savedStateHandle.optionalNftAssetId(RouteArgument.NftAssetId)

    private val nftAsset: Deferred<NFTAsset?> = viewModelScope.async(Dispatchers.IO, CoroutineStart.LAZY) {
        val id = nftAssetId ?: return@async null
        runCatching {
            getAssetNft.getAssetNft(id).first().assets.firstOrNull()
        }.getOrNull()
    }

    val state: StateFlow<RecipientState> = getAssetInfo(assetId)
        .filterNotNull()
        .map { assetInfo ->
            val type: RecipientType? = if (nftAssetId == null) {
                RecipientType.Asset(assetInfo)
            } else {
                nftAsset.await()?.let { RecipientType.Nft(assetInfo, it) }
            }
            type?.let(RecipientState::Ready) ?: RecipientState.Loading
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, RecipientState.Loading)

    val wallets = session.combine(getWallets()) { session, wallets ->
        wallets.filter { it.id != session?.wallet?.id }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val contacts: StateFlow<List<ContactRecipient>> = state
        .flatMapLatest { state ->
            when (state) {
                RecipientState.Loading -> flowOf(emptyList())
                is RecipientState.Ready -> getContacts.getContactRecipients(state.type.assetInfo.asset.chain)
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val addressError = combine(
        state,
        address,
        nameRecord,
    ) { state, address, record ->
        when (state) {
            RecipientState.Loading -> RecipientError.None
            is RecipientState.Ready -> {
                val asset = state.type.assetInfo.asset
                val resolvedAddress = asset.chain.checksumAddress(record?.address ?: address)
                when {
                    resolvedAddress.isEmpty() -> RecipientError.None
                    resolveName.canResolveName(address) -> RecipientError.None
                    else -> validateDestination(
                        asset = asset,
                        destination = DestinationAddress(resolvedAddress, record?.name),
                    )
                }
            }
        }
    }.mutableStateIn(viewModelScope, RecipientError.None)

    val memoErrorState = MutableStateFlow<RecipientError>(RecipientError.None)

    val hasMemo: StateFlow<Boolean> = state
        .map {
            when (it) {
                RecipientState.Loading -> false
                is RecipientState.Ready -> it.type.assetInfo.asset.chain.isMemoSupport()
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    fun onNext(
        type: RecipientType,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        submit(
            type = type,
            destination = DestinationAddress(
                address = nameRecord.value?.address ?: address.value,
                name = nameRecord.value?.name,
            ),
            amountAction = amountAction,
            confirmAction = confirmAction,
        )
    }

    fun onDestination(
        type: RecipientType,
        destination: DestinationAddress,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        submit(type, destination, amountAction, confirmAction)
    }

    private fun submit(
        type: RecipientType,
        destination: DestinationAddress,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        val asset = type.assetInfo.asset
        destination.copy(address = asset.chain.checksumAddress(destination.address)).let { destination ->
            val validation = validateDestination(asset, destination)
            if (validation != RecipientError.None) {
                if (!resolveName.canResolveName(destination.address)) {
                    addressError.update { validation }
                }
                return
            }
            when (type) {
                is RecipientType.Nft -> onNftConfirm(type.nftAsset, destination, confirmAction)
                is RecipientType.Asset -> amountAction(
                    AmountParams.Transfer(type.assetInfo.id(), destination, memo.value)
                )
            }
        }
    }

    fun onAddress(input: String, record: NameRecord?) {
        _address.value = input
        nameRecord.value = record
    }

    fun onMemo(input: String) {
        _memo.value = input
    }

    fun setQrData(type: RecipientType, field: QrScanField, data: String, confirmAction: ConfirmTransactionAction) {
        val paymentWrapper = uniffi.gemstone.paymentDecodeUrl(data)
        val amount = try {
            BigInteger(paymentWrapper.amount ?: throw IllegalArgumentException())
        } catch (_: Throwable) {
            null
        }
        val assetInfo = type.assetInfo
        val address = assetInfo.asset.chain.checksumAddress(paymentWrapper.address)
        val memo = paymentWrapper.memo

        val owner = assetInfo.owner
        if (
            address.isNotEmpty()
            && amount != null
            && owner != null
            && (assetInfo.asset.chain.isMemoSupport() || !memo.isNullOrEmpty())
        ) {
            val params = ConfirmParams.Builder(assetInfo.asset, owner, amount, false).transfer(DestinationAddress(address), memo)
            confirmAction(params)
            return
        }

        when (field) {
            QrScanField.None -> Unit
            QrScanField.Address -> {
                _address.value = address.ifEmpty { data }
                _memo.value = memo?.ifEmpty { _memo.value } ?: _memo.value
            }
            QrScanField.Memo -> {
                _address.value = address.ifEmpty { _address.value }
                _memo.value = paymentWrapper.memo ?: data
            }
        }
    }

    private fun onNftConfirm(nftAsset: NFTAsset, destination: DestinationAddress, confirmAction: ConfirmTransactionAction) {
        val params = ConfirmParams.NftParams(
            asset = nftAsset.chain.asset(),
            from = session.value?.wallet?.getAccount(nftAsset.chain) ?: return,
            destination = destination,
            nftAsset = nftAsset,
        )
        confirmAction(params)
    }

    private fun validateDestination(asset: Asset, destination: DestinationAddress): RecipientError =
        if (validateAddressOperator(destination.address, asset.chain).getOrNull() == true) {
            RecipientError.None
        } else {
            RecipientError.IncorrectAddress(asset.name)
        }
}
