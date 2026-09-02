package com.gemwallet.android.features.recipient.viewmodel

import uniffi.gemstone.GemRecipient
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.PaymentRecipient
import com.gemwallet.android.model.toPaymentWalletAsset
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.name.NameRecordState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.optionalNftAssetId
import com.gemwallet.android.ui.models.navigation.optionalPaymentRecipient
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.NameRecord
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemPaymentDestination
import uniffi.gemstone.GemRecipientException
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemRecipientServiceInterface
import uniffi.gemstone.GemTransactionInputType
import uniffi.gemstone.GemstoneException
import uniffi.gemstone.GemTransferData
import com.gemwallet.android.domains.confirm.confirmInput
import com.gemwallet.android.domains.confirm.transferNft
import java.math.BigInteger
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
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecipientViewModel @Inject constructor(
    private val getSession: GetSession,
    private val getContacts: GetContacts,
    private val getAssetInfo: GetAssetInfo,
    private val getAssetNft: GetAssetNft,
    savedStateHandle: SavedStateHandle,
    private val service: GemRecipientServiceInterface,
    nameService: GemNameServiceInterface,
) : ViewModel() {

    private val addressInput = AddressInputModel(nameService, viewModelScope)

    val address: StateFlow<String> = addressInput.text
    val nameResolveState: StateFlow<NameRecordState> = addressInput.nameResolveState
    val addressError: StateFlow<Boolean> = addressInput.showError

    private val _memo = MutableStateFlow("")
    val memo = _memo.asStateFlow()
    private var references = emptyList<String>()
    private var requestedAmount: String? = null

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

    val wallets = session.map { session ->
        session?.wallet?.let { wallet -> service.otherWallets(wallet.id.id).map { it.decodeJson<Wallet>() } }.orEmpty()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val contacts: StateFlow<List<ContactRecipient>> = state
        .flatMapLatest { state ->
            when (state) {
                RecipientState.Loading -> flowOf(emptyList())
                is RecipientState.Ready -> getContacts.getContactRecipients(state.type.assetInfo.asset.chain)
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val memoErrorState = MutableStateFlow<RecipientError>(RecipientError.None)

    val buttonState: StateFlow<ButtonState> = addressInput.isValid
        .map { buttonState(enabled = it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    init {
        viewModelScope.launch {
            state.filterIsInstance<RecipientState.Ready>()
                .collect { addressInput.setChain(it.type.assetInfo.asset.chain) }
        }
    }

    val hasMemo: StateFlow<Boolean> = state
        .map {
            when (it) {
                RecipientState.Loading -> false
                is RecipientState.Ready -> it.type.assetInfo.asset.chain.isMemoSupport()
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    init {
        savedStateHandle.optionalPaymentRecipient(RouteArgument.Payment)?.let(::updateFrom)
    }

    fun onNext(
        type: RecipientType,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        if (!addressInput.validate()) return
        submit(type, address.value, addressInput.nameRecord, amountAction, confirmAction)
    }

    fun onDestination(
        type: RecipientType,
        destination: GemRecipient,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
    ) {
        submit(type, destination.address, null, amountAction, confirmAction, destination.name)
    }

    private fun submit(
        type: RecipientType,
        input: String,
        nameRecord: NameRecord?,
        amountAction: AmountTransactionAction,
        confirmAction: ConfirmTransactionAction,
        selectedName: String? = null,
    ) {
        val chain = type.assetInfo.asset.chain
        val recipient = try {
            service.recipient(chain.string, input, nameRecord?.toJson(), memo.value, references)
        } catch (_: GemRecipientException) {
            addressInput.markInvalid()
            return
        }
        val destination = GemRecipient(address = recipient.address, name = recipient.name ?: selectedName)
        when (type) {
            is RecipientType.Nft -> onNftConfirm(type.nftAsset, destination, confirmAction)
            is RecipientType.Asset -> amountAction(
                AmountParams.Transfer(type.assetInfo.id(), destination, memo.value, references, requestedAmount)
            )
        }
    }

    fun onAddress(input: String) {
        if (input != address.value) {
            requestedAmount = null
            references = emptyList()
        }
        addressInput.onTextChange(input)
    }

    fun onMemo(input: String) {
        _memo.value = input
    }

    fun setQrData(type: RecipientType, field: QrScanField, data: String, confirmAction: ConfirmTransactionAction) {
        when (field) {
            QrScanField.None -> Unit
            QrScanField.Memo -> _memo.value = data
            QrScanField.Address -> onAddressScan(type, data, confirmAction)
        }
    }

    private fun onAddressScan(type: RecipientType, data: String, confirmAction: ConfirmTransactionAction) {
        val asset = type.assetInfo.asset
        val destination = try {
            service.scanDestination(data, type.assetInfo.toPaymentWalletAsset())
        } catch (_: GemstoneException) {
            addressInput.markInvalid()
            return
        }
        when (destination) {
            is GemPaymentDestination.Confirm -> {
                val transfer = service.transferData(destination.transfer, asset.toGem())
                when (type) {
                    is RecipientType.Nft -> updateFrom(PaymentRecipient(transfer.recipient, null))
                    is RecipientType.Asset -> session.value?.wallet?.getAccount(asset.chain)?.let { confirmAction(transfer.confirmInput(it)) }
                }
            }
            is GemPaymentDestination.Recipient -> updateFrom(PaymentRecipient(destination.recipient, destination.amount))
            is GemPaymentDestination.SelectAsset, is GemPaymentDestination.Unsupported -> addressInput.markInvalid()
        }
    }

    private fun updateFrom(payment: PaymentRecipient) {
        addressInput.applyExternalAddress(payment.recipient.address)
        payment.recipient.memo?.let { _memo.value = it }
        references = payment.recipient.references
        requestedAmount = payment.amount
    }

    private fun onNftConfirm(nftAsset: NFTAsset, destination: GemRecipient, confirmAction: ConfirmTransactionAction) {
        val from = session.value?.wallet?.getAccount(nftAsset.chain) ?: return
        val transfer = GemTransferData(
            inputType = GemTransactionInputType.transferNft(nftAsset.chain.asset(), nftAsset),
            recipient = destination,
            value = BigInteger.ZERO.toString(),
        )
        confirmAction(transfer.confirmInput(from))
    }
}
