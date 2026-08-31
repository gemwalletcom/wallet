package com.gemwallet.android.features.recipient.viewmodel

import uniffi.gemstone.GemAddressService
import uniffi.gemstone.GemRecipient
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.wallet.cases.GetWallets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.contacts.values.ContactRecipient
import com.gemwallet.android.application.contacts.cases.GetContacts
import com.gemwallet.android.application.recipient.cases.GetNameRecord
import com.gemwallet.android.application.nft.cases.GetAssetNft
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.ext.decodePayment
import com.gemwallet.android.ext.exactAmount
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.request
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientState
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.PaymentDestination
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.name.AddressInputModel
import com.gemwallet.android.ui.models.name.NameRecordState
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.optionalNftAssetId
import com.gemwallet.android.ui.models.navigation.optionalPaymentRequest
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.NameRecord
import uniffi.gemstone.GemRecipientException
import uniffi.gemstone.GemPaymentService
import uniffi.gemstone.GemRecipientService
import com.wallet.core.primitives.PaymentRequest
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
    private val getWallets: GetWallets,
    private val getContacts: GetContacts,
    private val getAssetInfo: GetAssetInfo,
    private val getAssetNft: GetAssetNft,
    private val getNameRecord: GetNameRecord,
    savedStateHandle: SavedStateHandle,
    private val recipientService: GemRecipientService,
    private val paymentService: GemPaymentService,
    private val addressService: GemAddressService,
) : ViewModel() {

    private val addressInput = AddressInputModel(
        getNameRecord = getNameRecord,
        recipientService = recipientService,
        scope = viewModelScope,
    )

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
        savedStateHandle.optionalPaymentRequest(RouteArgument.Payment)?.let(::updateFrom)
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
            recipientService.recipient(chain.string, input, nameRecord?.toJson(), memo.value, references)
        } catch (_: GemRecipientException) {
            if (!getNameRecord.isNameSupported(input)) {
                addressInput.markInvalid()
            }
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
        when (val destination = scannedDestination(type, data)) {
            PaymentDestination.Unsupported -> addressInput.markInvalid()
            is PaymentDestination.Confirm -> confirmAction(destination.params)
            is PaymentDestination.Recipient -> updateFrom(destination.request)
        }
    }

    private fun scannedDestination(type: RecipientType, data: String): PaymentDestination.Transfer {
        val request = paymentService.decodePayment(data)?.request ?: return PaymentDestination.Unsupported

        return when (type) {
            is RecipientType.Nft -> PaymentDestination.Recipient(type.assetInfo.asset.id, request.copy(amount = null))
            is RecipientType.Asset -> PaymentDestination.transfer(request, type.assetInfo, paymentService)
        }
    }

    private fun updateFrom(request: PaymentRequest) {
        addressInput.applyExternalAddress(assetId.chain.checksumAddress(request.address, addressService))
        request.memo?.let { _memo.value = it }
        references = request.references.orEmpty()
        requestedAmount = request.exactAmount
    }

    private fun onNftConfirm(nftAsset: NFTAsset, destination: GemRecipient, confirmAction: ConfirmTransactionAction) {
        val params = ConfirmParams.NftParams(
            asset = nftAsset.chain.asset(),
            from = session.value?.wallet?.getAccount(nftAsset.chain) ?: return,
            destination = destination,
            nftAsset = nftAsset,
        )
        confirmAction(params)
    }
}
