package com.gemwallet.android.features.recipient.viewmodel

import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.cases.nft.GetAssetNft
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.isMemoSupport
import com.gemwallet.android.ext.mutableStateIn
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.features.recipient.viewmodel.models.QrScanField
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientError
import com.gemwallet.android.features.recipient.viewmodel.models.RecipientType
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NameRecord
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import java.math.BigInteger
import javax.inject.Inject

const val assetIdArg = "assetId"
const val nftAssetIdArg = "nftAssetId"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecipientViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val walletsRepository: WalletsRepository,
    private val assetsRepository: AssetsRepository,
    private val getAssetNft: GetAssetNft,
    private val validateAddressOperator: ValidateAddressOperator,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    val addressState = mutableStateOf("")
    val memoState = mutableStateOf("")
    val nameRecordState = mutableStateOf<NameRecord?>(null)

    private val session = sessionRepository.session()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val assetId = savedStateHandle.getStateFlow(assetIdArg, "")
        .mapNotNull { it.toAssetId() }
    private val nftAssetId: StateFlow<String?> = savedStateHandle.getStateFlow(nftAssetIdArg, null)

    val type: StateFlow<RecipientType?> = combine(
        assetId.flatMapLatest { assetsRepository.getAssetInfo(it) }.flowOn(Dispatchers.IO),
        nftAssetId,
        ::Pair,
    ).flatMapLatest { (assetInfo, nftId) ->
        when {
            assetInfo == null -> flowOf(null)
            nftId.isNullOrEmpty() -> flowOf(RecipientType.Asset(assetInfo))
            else -> getAssetNft.getAssetNft(nftId).map { data ->
                data.assets.firstOrNull()?.let { RecipientType.Nft(assetInfo, it) }
            }
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val wallets = session.combine(walletsRepository.getAll()) { session, wallets ->
        wallets.filter { it.id != session?.wallet?.id }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val addressError = combine(
        type,
        snapshotFlow { addressState.value },
        snapshotFlow { nameRecordState.value },
    ) { currentType, address, nameRecord ->
        val resolvedAddress = nameRecord?.address ?: address
        when {
            currentType == null || resolvedAddress.isEmpty() -> RecipientError.None
            else -> validateDestination(
                chain = currentType.assetInfo.asset.chain,
                destination = DestinationAddress(resolvedAddress, nameRecord?.name),
            )
        }
    }.mutableStateIn(viewModelScope, RecipientError.None)

    val memoErrorState = MutableStateFlow<RecipientError>(RecipientError.None)

    fun hasMemo(): Boolean = type.value?.assetInfo?.asset?.chain?.isMemoSupport() == true

    fun onNext(destination: DestinationAddress?, amountAction: AmountTransactionAction, confirmAction: ConfirmTransactionAction) {
        val currentType = type.value ?: return
        val resolvedDestination = destination ?: DestinationAddress(
            address = nameRecordState.value?.address ?: addressState.value,
            name = nameRecordState.value?.name,
        )
        val validation = validateDestination(currentType.assetInfo.asset.chain, resolvedDestination)
        if (validation != RecipientError.None) {
            addressError.update { validation }
            return
        }
        when (currentType) {
            is RecipientType.Nft -> onNftConfirm(currentType.nftAsset, resolvedDestination, confirmAction)
            is RecipientType.Asset -> amountAction(
                AmountParams.buildTransfer(currentType.assetInfo.id(), resolvedDestination, memoState.value)
            )
        }
    }

    fun setQrData(field: QrScanField, data: String, confirmAction: ConfirmTransactionAction) {
        val paymentWrapper = uniffi.gemstone.paymentDecodeUrl(data)
        val amount = try {
            BigInteger(paymentWrapper.amount ?: throw IllegalArgumentException())
        } catch (_: Throwable) {
            null
        }
        val address = paymentWrapper.address
        val memo = paymentWrapper.memo
        val assetInfo = type.value?.assetInfo ?: return

        if (
            address.isNotEmpty()
            && amount != null
            && (assetInfo.asset.chain.isMemoSupport() || !memo.isNullOrEmpty())
        ) {
            val params = ConfirmParams.Builder(assetInfo.asset, assetInfo.owner!!, amount, false).transfer(DestinationAddress(address), memo)
            confirmAction(params)
            return
        }

        when (field) {
            QrScanField.None -> Unit
            QrScanField.Address -> {
                addressState.value = address.ifEmpty { data }
                memoState.value = memo?.ifEmpty { memoState.value } ?: memoState.value
            }
            QrScanField.Memo -> {
                addressState.value = address.ifEmpty { addressState.value }
                memoState.value = paymentWrapper.memo ?: data
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

    private fun validateDestination(chain: Chain, destination: DestinationAddress?): RecipientError =
        if (validateAddressOperator(destination?.address.orEmpty(), chain).getOrNull() == true) {
            RecipientError.None
        } else {
            RecipientError.IncorrectAddress
        }
}