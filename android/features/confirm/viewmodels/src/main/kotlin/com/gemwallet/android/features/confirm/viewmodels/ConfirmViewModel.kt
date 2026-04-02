package com.gemwallet.android.features.confirm.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.LoadPrivateKeyOperator
import com.gemwallet.android.blockchain.services.BroadcastService
import com.gemwallet.android.blockchain.services.SignClientProxy
import com.gemwallet.android.blockchain.services.SignerPreloaderProxy
import com.gemwallet.android.cases.transactions.CreateTransaction
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.stake.sumRewardsBalance
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.isMemoSupport
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.math.MAX_256
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.GemPlatformErrors
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.SignerParams
import com.gemwallet.android.model.format
import com.gemwallet.android.model.getDelegatePreparedAmount
import com.gemwallet.android.serializer.jsonEncoder
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelFactory
import com.gemwallet.android.ui.models.swap.SwapDetailsUIModelInput
import com.gemwallet.android.ui.models.swap.SwapProviderUIModelFactory
import com.gemwallet.android.ui.models.actions.FinishConfirmAction
import com.gemwallet.android.ui.models.navigation.assetRoutePath
import com.gemwallet.android.ui.models.navigation.stakeRoute
import com.gemwallet.android.ui.models.navigation.swapRoute
import com.gemwallet.android.features.confirm.models.AmountUIModel
import com.gemwallet.android.features.confirm.models.ConfirmDetailElement
import com.gemwallet.android.features.confirm.models.ConfirmError
import com.gemwallet.android.features.confirm.models.ConfirmProperty
import com.gemwallet.android.features.confirm.models.ConfirmState
import com.gemwallet.android.features.confirm.models.FeeUIModel
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionNFTTransferMetadata
import com.wallet.core.primitives.TransactionSwapMetadata
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.gemwallet.android.ext.getMinimumAccountBalance
import com.wallet.core.primitives.SimulationResult
import java.math.BigInteger
import java.util.Arrays
import javax.inject.Inject

internal const val paramsArg = "data"
internal const val txTypeArg = "tx_type"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConfirmViewModel @Inject constructor(
    private val sessionRepository: SessionRepository,
    private val assetsRepository: AssetsRepository,
    private val signerPreload: SignerPreloaderProxy,
    private val passwordStore: PasswordStore,
    private val loadPrivateKeyOperator: LoadPrivateKeyOperator,
    private val signClient: SignClientProxy,
    private val broadcastService: BroadcastService,
    private val createTransactionsCase: CreateTransaction,
    private val stakeRepository: StakeRepository,
    private val transactionBalanceService: TransactionBalanceService,
    private val savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val restart = MutableStateFlow(false)
    val state = MutableStateFlow<ConfirmState>(ConfirmState.Prepare)
    val feePriority = MutableStateFlow(FeePriority.Normal)
    private val walletConnectSimulationState = MutableStateFlow<SimulationResult?>(null)

    private val request = savedStateHandle.getStateFlow<String?>(paramsArg, null)
        .combine(restart) { request, _ -> request }
        .filterNotNull()
        .mapNotNull { paramsPack ->
            state.update { ConfirmState.Prepare }
            ConfirmParams.unpack(paramsPack)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val session = sessionRepository.session()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val walletConnectHeaderAssetInfo = walletConnectSimulationState
        .map { it?.header?.assetId }
        .distinctUntilChanged()
        .flatMapLatest { assetId ->
            if (assetId == null) return@flatMapLatest flowOf(null)
            assetsRepository.getTokenInfo(assetId).also {
                if (assetId.tokenId != null && it.firstOrNull() == null) {
                    assetsRepository.searchToken(assetId, sessionRepository.getCurrentCurrency())
                }
            }
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletConnectReview = combine(walletConnectSimulationState, walletConnectHeaderAssetInfo, request) { simulation, headerAssetInfo, params ->
        val review = simulation?.toWalletConnectReview() ?: WalletConnectReview()
        val headerAssetId = simulation?.header?.assetId
        val asset = when {
            headerAssetId == null || params == null -> null
            headerAssetId == params.assetId -> params.asset
            else -> headerAssetInfo?.asset
        }
        review.copy(headerAsset = asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, WalletConnectReview())

    private val assetsInfo = request.filterNotNull().mapNotNull {
        if (it is ConfirmParams.SwapParams) {
            listOf(it.fromAsset.id, it.toAsset.id)
        } else {
            listOf(it.assetId)
        }
    }
    .flatMapLatest { assetsRepository.getAssetsInfo(it) }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val preloadData = combine(
        session,
        request.filterNotNull()
    ) { session, request ->
        val owner = session?.wallet?.getAccount(request.assetId.chain)
        if (owner == null) {
            state.update { ConfirmState.FatalError("Session not found") }
            return@combine null
        }

        val preload = try {
            signerPreload.preload(params = request)
        } catch (err: Throwable) {
            state.update {
                ConfirmState.Error(
                    when (err.message?.contains(GemPlatformErrors.Dust.message)) {
                        true -> ConfirmError.DustThreshold("${owner.chain.asset().name} (${owner.chain.asset().symbol})")
                        else -> ConfirmError.PreloadError
                    }
                )
            }
            return@combine null
        }
        preload
    }
    .filterNotNull()
    .combine(feePriority) { params, feePriority ->
        val energyAsset = params.input.assetId == params.fee(feePriority).feeAssetId
        val finalAmount = when {
            params.input is ConfirmParams.Stake.RewardsParams -> stakeRepository.getRewards(params.input.assetId, params.input.from.address)
                .sumRewardsBalance()

            params.input.useMaxAmount && energyAsset -> {
                val result = params.input.amount - params.fee(feePriority).amount
                result
            }
            else -> params.input.amount
        }
        state.update { ConfirmState.Ready }

        if (params.input is ConfirmParams.SwapParams) {
            assembleMetadata(params)
        }

        params.copy(finalAmount = finalAmount)
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val feeAssetInfo = preloadData.filterNotNull().flatMapLatest { signerParams ->
        assetsRepository.getAssetInfo(signerParams.fee().feeAssetId)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val amountUIModel = combine(request, assetsInfo, preloadData) { request, assetsInfo, signerParams ->
        val fromAssetId = request?.assetId ?: return@combine null
        val assetInfo = assetsInfo?.getByAssetId(fromAssetId) ?: return@combine null
        val toAssetInfo = if (request is ConfirmParams.SwapParams) {
            assetsInfo.getByAssetId(request.toAsset.id) ?: return@combine null
        } else {
            null
        }

        val amount = Crypto(signerParams?.finalAmount ?: request.amount)
        val price = assetInfo.price?.price?.price ?: 0.0
        val currency = assetInfo.price?.currency ?: Currency.USD
        val decimals = assetInfo.asset.decimals
        val symbol = assetInfo.asset.symbol

        AmountUIModel(
            txType = request.getTxType(),
            amount = amount.format(decimals, symbol, -1),
            amountEquivalent = currency.format(amount.convert(decimals, price).atomicValue, dynamicPlace = true),
            asset = assetInfo,
            fromAsset = assetInfo,
            fromAmount = amount.atomicValue.toString(),
            toAsset = toAssetInfo,
            toAmount = (request as? ConfirmParams.SwapParams)?.toAmount?.toString(),
            nftAsset = (request as? ConfirmParams.NftParams)?.nftAsset,
            currency = currency,
        )
    }
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val detailElements = combine(request, assetsInfo, ::buildDetailElements)
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val txProperties = combine(request, assetsInfo) { request, assetsInfo ->
        request ?: return@combine emptyList()
        val assetInfo = assetsInfo?.getByAssetId(request.assetId) ?: return@combine emptyList()
        mutableListOf<ConfirmProperty?>().apply {
            add(ConfirmProperty.Source(assetInfo.walletName))
            add(ConfirmProperty.Destination.map(request, getValidator(request)))
            add(request.memo()?.takeIf {
                (request is ConfirmParams.TransferParams.Native || request is ConfirmParams.TransferParams.Token)
                        && assetInfo.asset.isMemoSupport()
                        && it.isNotEmpty()
            }?.let { ConfirmProperty.Memo(it) })
            add(ConfirmProperty.Network(assetInfo.chain.asset()))
        }.filterNotNull()
    }
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val feeValue = combine(preloadData, feeAssetInfo, state, feePriority) { signerParams, feeAssetInfo, _, feePriority ->
        val amount = signerParams?.fee(feePriority)?.amount
        if (amount == null || feeAssetInfo == null) {
            return@combine ""
        }
        val feeAmount = Crypto(amount)
        feeAssetInfo.asset.format(feeAmount, 8, dynamicPlace = true)
    }
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val feeUIModel = combine(preloadData, feeAssetInfo, state, feePriority) { signerParams, feeAssetInfo, state, priority ->
        val amount = signerParams?.fee(priority)?.amount
        val result = if (amount == null || feeAssetInfo == null) {
            if (state is ConfirmState.Error) FeeUIModel.Error else FeeUIModel.Calculating
        } else {
            val feeAmount = Crypto(amount)
            val currency = feeAssetInfo.price?.currency ?: Currency.USD
            val feeDecimals = feeAssetInfo.asset.decimals
            val feeCrypto = feeAssetInfo.asset.format(feeAmount, 8, dynamicPlace = true)
            val feeFiat = feeAssetInfo.price?.let {
                currency.format(feeAmount.convert(feeDecimals, it.price.price).atomicValue, dynamicPlace = true) // TODO: Move to UI - Model
            } ?: ""

            try {
                val sendAssetInfo = assetsInfo.value?.getByAssetId(signerParams.input.assetId)
                if (sendAssetInfo != null) {
                    validateBalance(
                        signerParams,
                        feePriority.value,
                        sendAssetInfo,
                        feeAssetInfo,
                        getBalance(sendAssetInfo, signerParams.input)
                    )
                }
            } catch (err: ConfirmError) {
                this@ConfirmViewModel.state.update { ConfirmState.Error(err) }
            }

            FeeUIModel.FeeInfo(
                cryptoAmount = feeCrypto,
                fiatAmount = feeFiat,
                feeAsset = feeAssetInfo.asset,
                priority = priority,
            )
        }
        result
    }
    .flowOn(Dispatchers.Default)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val allFee = preloadData.filterNotNull().map { it.allFee() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun init(params: ConfirmParams, walletConnectSimulation: SimulationResult? = null) {
        viewModelScope.launch(Dispatchers.IO) {
            state.update { ConfirmState.Prepare }
            walletConnectSimulationState.value = walletConnectSimulation
            // reset
            savedStateHandle[txTypeArg] = null
            savedStateHandle[paramsArg] = null
            // load
            savedStateHandle[txTypeArg] = params.getTxType().string
            savedStateHandle[paramsArg] = params.pack()
        }
    }

    fun changeFeePriority(feePriority: FeePriority) {
        if (feePriority == this.feePriority.value) {
            return
        }
        state.update { ConfirmState.Prepare }
        this.feePriority.update { feePriority }
    }

    fun send(finishAction: FinishConfirmAction) = viewModelScope.launch(Dispatchers.IO) {
        if (state.value is ConfirmState.Error) {
            restart.update { !it }
            return@launch
        }
        state.update { ConfirmState.Sending }

        val signerParams = preloadData.value
        val assetInfo = assetsInfo.value?.getByAssetId(signerParams?.input?.assetId ?: return@launch)
        val account = assetInfo?.owner
        val feeAssetInfo = feeAssetInfo.value
        val session = session.value
        val feePriority = this@ConfirmViewModel.feePriority.value

        try {
            if (assetInfo == null || account == null || session == null || feeAssetInfo == null) {
                throw ConfirmError.TransactionIncorrect
            }
            validateBalance(
                signerParams,
                feePriority,
                assetInfo,
                feeAssetInfo,
                getBalance(assetInfo, signerParams.input),
            )
            val signs = sign(signerParams, session, assetInfo, feePriority)
            if (signs.isEmpty()) {
                throw IllegalStateException("Not implemented")
            }
            when (signerParams.input) {
                is ConfirmParams.TransferParams.Generic -> {
                    if (!(signerParams.input as ConfirmParams.TransferParams.Generic).isSendable) {
                        val hash = String(signs.firstOrNull() ?: byteArrayOf())
                        state.update { ConfirmState.Result(txHash = hash) }
                        viewModelScope.launch(Dispatchers.Main) {
                            finishAction(assetId = assetInfo.id(), hash = hash, route = "")
                        }
                        return@launch
                    }
                }
                else -> {}
            }
            for (sign in signs) {
                val txHash = broadcastService.send(account, sign, signerParams.input.getTxType())
                if (!sign.contentEquals(signs.last())) {
                    delay(500)
                } else {
                    addTransaction(txHash)
                    addRecent(assetInfo, signerParams.input)
                    val finishRoute = when (signerParams.input) {
                        is ConfirmParams.Stake -> stakeRoute
                        is ConfirmParams.SwapParams,
                        is ConfirmParams.TokenApprovalParams -> swapRoute
                        is ConfirmParams.TransferParams -> assetRoutePath
                        is ConfirmParams.Activate -> assetRoutePath
                        is ConfirmParams.NftParams -> assetRoutePath
                        is ConfirmParams.PerpetualParams -> assetRoutePath // TODO: Move to perpetuals
                    }
                    viewModelScope.launch(Dispatchers.Main) {
                        finishAction(assetId = assetInfo.id(), hash = txHash, route = finishRoute)
                    }
                    state.update { ConfirmState.Result(txHash = txHash) }
                }
            }
        } catch (err: ConfirmError) {
            state.update { ConfirmState.BroadcastError(err) }
        } catch (err: Throwable) {
            state.update { ConfirmState.BroadcastError(ConfirmError.BroadcastError(err.message ?: "Unknown error")) }
        }
    }

    fun addRecent(
        assetInfo: AssetInfo,
        request: ConfirmParams
    ) {
        val walletId = assetInfo.walletId ?: return
        val type = when (request) {
            is ConfirmParams.SwapParams -> RecentType.Swap
            is ConfirmParams.TransferParams -> RecentType.Send
            else -> return
        }
        val toAssetId = if (request is ConfirmParams.SwapParams) {
            request.toAsset.id
        } else {
            null
        }
        viewModelScope.launch(Dispatchers.IO) {
            try {
                assetsRepository.addRecentActivity(assetInfo.id(), walletId, type, toAssetId)
            } catch (_: Throwable) {}
        }
    }

    private suspend fun  sign(signerParams: SignerParams, session: Session, assetInfo: AssetInfo, feePriority: FeePriority): List<ByteArray> {
        val key = loadPrivateKeyOperator(
            session.wallet,
            assetInfo.id().chain,
            passwordStore.getPassword(session.wallet.id)
        )
        val sign = try {
            signClient.signTransaction(
                params = signerParams,
                feePriority = feePriority,
                privateKey = key
            )
        } catch (_: Throwable) {
            throw ConfirmError.SignFail
        } finally {
            Arrays.fill(key, 0)
        }
        return sign
    }

    private suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        return transactionBalanceService.getBalance(assetInfo, params)
    }

    private suspend fun getValidator(params: ConfirmParams): DelegationValidator? {
        val validatorId = when (params) {
            is ConfirmParams.Stake.DelegateParams -> params.validator.id
            is ConfirmParams.Stake.RedelegateParams -> params.dstValidator.id
            is ConfirmParams.Stake.UndelegateParams -> params.delegation.base.validatorId
            is ConfirmParams.Stake.WithdrawParams -> params.delegation.base.validatorId
            is ConfirmParams.Activate,
            is ConfirmParams.Stake.RewardsParams,
            is ConfirmParams.Stake.Freeze,
            is ConfirmParams.Stake.Unfreeze,
            is ConfirmParams.SwapParams,
            is ConfirmParams.TokenApprovalParams,
            is ConfirmParams.NftParams,
            is ConfirmParams.PerpetualParams,
            is ConfirmParams.TransferParams -> null
        }
        return stakeRepository.getStakeValidator(params.assetId, validatorId ?: return null)
    }

    private fun List<AssetInfo>.getByAssetId(assetId: AssetId): AssetInfo? {
        val str = assetId.toIdentifier()
        return firstOrNull { it.id().toIdentifier() ==  str}
    }

    private fun buildDetailElements(
        request: ConfirmParams?,
        assetsInfo: List<AssetInfo>?,
    ): List<ConfirmDetailElement> {
        return listOfNotNull(
            buildSwapDetailElement(request as? ConfirmParams.SwapParams, assetsInfo),
        )
    }

    private fun buildSwapDetailElement(
        params: ConfirmParams.SwapParams?,
        assetsInfo: List<AssetInfo>?,
    ): ConfirmDetailElement.SwapDetails? {
        val params = params ?: return null
        val assetsInfo = assetsInfo ?: return null
        val fromAssetInfo = assetsInfo.getByAssetId(params.fromAsset.id) ?: return null
        val toAssetInfo = assetsInfo.getByAssetId(params.toAsset.id) ?: return null

        val provider = SwapProviderUIModelFactory.create(
            providerId = params.providerId,
            title = params.protocol,
            receiveAsset = toAssetInfo,
            toValue = params.toAmount.toString(),
        )
        val model = SwapDetailsUIModelFactory.create(
            SwapDetailsUIModelInput(
                payAsset = fromAssetInfo,
                receiveAsset = toAssetInfo,
                fromValue = params.fromAmount.toString(),
                toValue = params.toAmount.toString(),
                provider = provider,
                slippageBps = params.slippageBps,
                etaInSeconds = params.etaInSeconds,
                isProviderSelectable = false,
            )
        ) ?: return null

        return ConfirmDetailElement.SwapDetails(model)
    }

    private fun assembleMetadata(signerParams: SignerParams) = when (val input = signerParams.input) {
        is ConfirmParams.SwapParams -> {
            jsonEncoder.encodeToString(
                TransactionSwapMetadata(
                    fromAsset = input.fromAsset.id,
                    toAsset = input.toAsset.id,
                    fromValue = input.fromAmount.toString(),
                    toValue = input.toAmount.toString(),
                    provider = input.protocolId,
                )
            )
        }
        is ConfirmParams.NftParams -> jsonEncoder.encodeToString(
            TransactionNFTTransferMetadata(input.nftAsset.id, input.nftAsset.name)
        )
        else -> null
    }

    private suspend fun addTransaction(txHash: String) {
        val signerParams = preloadData.value
        val assetInfo = assetsInfo.value?.getByAssetId(signerParams?.input?.assetId ?: return) ?: return
        val session = session.value
        val destinationAddress =  signerParams.input.destination()?.address ?: ""
        val priority = feePriority.value

        createTransactionsCase.createTransaction(
            hash = txHash,
            walletId = session?.wallet?.id ?: return,
            assetId = assetInfo.id(),
            owner = assetInfo.owner!!,
            to = destinationAddress,
            state = TransactionState.Pending,
            fee = signerParams.fee(priority),
            amount = signerParams.finalAmount,
            memo = signerParams.input.memo() ?: "",
            type = signerParams.input.getTxType(),
            metadata = assembleMetadata(signerParams),
            direction = if (destinationAddress == assetInfo.owner!!.address) {
                TransactionDirection.SelfTransfer
            } else {
                TransactionDirection.Outgoing
            },
            blockNumber = signerParams.data(feePriority.value).chainData.blockNumber()
        )
    }

    companion object {
        fun validateBalance(
            signerParams: SignerParams,
            feePriority: FeePriority,
            assetInfo: AssetInfo,
            feeAssetInfo: AssetInfo,
            assetBalance: BigInteger,
        ) {
            val amount = signerParams.finalAmount
            val feeAmount = signerParams.fee(feePriority).amount

            val totalAmount = when (signerParams.input.getTxType()) {
                TransactionType.Transfer,
                TransactionType.Swap,
                TransactionType.TokenApproval,
                TransactionType.AssetActivation,
                TransactionType.StakeFreeze -> amount + if (assetInfo == feeAssetInfo) feeAmount else BigInteger.ZERO
                TransactionType.EarnDeposit,
                TransactionType.StakeDelegate -> if (assetInfo.stakeChain?.freezed() == true) {
                    amount
                } else {
                    amount + if (assetInfo == feeAssetInfo) feeAmount else BigInteger.ZERO
                }
                TransactionType.StakeUndelegate,
                TransactionType.StakeRewards,
                TransactionType.StakeRedelegate,
                TransactionType.StakeWithdraw,
                TransactionType.EarnWithdraw,
                TransactionType.StakeUnfreeze,
                TransactionType.TransferNFT -> amount
                TransactionType.SmartContractCall -> TODO()
                TransactionType.PerpetualOpenPosition -> TODO()
                TransactionType.PerpetualClosePosition -> TODO()
                TransactionType.PerpetualModifyPosition -> TODO()
            }

            if (assetBalance < totalAmount) {
                val label = "${assetInfo.asset.name} (${assetInfo.asset.symbol})"
                throw ConfirmError.InsufficientBalance(label)
            }
            if (feeAssetInfo.balance.balance.available.toBigInteger() < feeAmount) {
                throw ConfirmError.InsufficientFee(chain = feeAssetInfo.asset.chain)
            }

            val minimumAssetBalance = assetInfo.chain.getMinimumAccountBalance()

            if (!signerParams.input.useMaxAmount
                && assetInfo.asset.type == AssetType.NATIVE
                && minimumAssetBalance > 0L
                && (feeAssetInfo.balance.balance.available.toBigInteger() - totalAmount).let { it > -MAX_256 && it < BigInteger.valueOf(minimumAssetBalance)}) {
                throw ConfirmError.MinimumAccountBalanceTooLow(asset = feeAssetInfo.asset, required = minimumAssetBalance)
            }
        }
    }
}
