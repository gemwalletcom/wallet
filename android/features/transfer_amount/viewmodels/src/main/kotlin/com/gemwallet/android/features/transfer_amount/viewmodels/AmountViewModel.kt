package com.gemwallet.android.features.transfer_amount.viewmodels

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.domains.stake.rewardsBalance
import com.gemwallet.android.domains.transaction.TransactionBalanceContext
import com.gemwallet.android.domains.transaction.balance
import com.gemwallet.android.ext.freezed
import com.gemwallet.android.math.parseNumber
import com.gemwallet.android.math.parseNumberOrNull
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.format
import com.gemwallet.android.ui.models.AmountInputType
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.Resource
import com.wallet.core.primitives.TransactionType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.launchIn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.Config
import java.math.BigInteger
import java.math.MathContext
import javax.inject.Inject

private const val paramsArg = "params"

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AmountViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val stakeRepository: StakeRepository,
    private val transactionBalanceService: TransactionBalanceService,
    savedStateHandle: SavedStateHandle
) : ViewModel() {

    val params = savedStateHandle
        .getStateFlow(paramsArg, "")
        .mapNotNull { AmountParams.unpack(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    var amount by mutableStateOf("")
        private set

    var amountInputType = MutableStateFlow(AmountInputType.Crypto)

    val errorUIState = MutableStateFlow<AmountError>(AmountError.None)

    private val maxAmount = MutableStateFlow(false)

    val resource = MutableStateFlow(Resource.Bandwidth)

    val assetInfo = params.flatMapLatest {
        it?.assetId?.let { assetId -> assetsRepository.getAssetInfo(assetId).filterNotNull() } ?: emptyFlow()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val reserveForFee = combine(params.filterNotNull(), assetInfo.filterNotNull(), maxAmount) { params, assetInfo, maxAmount ->
        if (!maxAmount) {
            return@combine null
        }
        val value = getReserveForFee(params.txType, assetInfo.asset.chain)
        if (value == BigInteger.ZERO) {
            return@combine null
        }
        assetInfo.asset.format(value, decimalPlace = 4)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)


    private val selectedValidatorId = MutableStateFlow<String?>(null)

    private val delegation: StateFlow<Delegation?> = combine(params, assetInfo, selectedValidatorId) { params, assetInfo, selectedId ->
        Triple(params, assetInfo, selectedId)
    }.flatMapLatest { (params, assetInfo, selectedId) ->
        when (params?.txType) {
            TransactionType.StakeUndelegate,
            TransactionType.StakeRedelegate,
            TransactionType.StakeWithdraw -> {
                val validatorId = params.validatorId ?: return@flatMapLatest flowOf(null)
                stakeRepository.getDelegation(validatorId, params.delegationId ?: "")
            }
            TransactionType.StakeRewards -> {
                val owner = assetInfo?.owner?.address ?: return@flatMapLatest flowOf(null)
                stakeRepository.getDelegations(assetInfo.asset.id, owner).map { list ->
                    val rewards = list.filter { it.hasRewards() }
                    rewards.firstOrNull { it.validator.id == selectedId } ?: rewards.firstOrNull()
                }
            }
            else -> flowOf(null)
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val recommendedValidator = params
        .flatMapLatest { params ->
            params?.assetId?.chain?.let { stakeRepository.getRecommended(params.assetId.chain) } ?: emptyFlow()
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val srcValidator = combine(params, delegation, recommendedValidator) { params, delegation, recommended ->
        when (params?.txType) {
            TransactionType.StakeWithdraw,
            TransactionType.StakeUndelegate,
            TransactionType.StakeRewards -> delegation?.validator
            TransactionType.StakeDelegate,
            TransactionType.StakeRedelegate -> recommended
            else -> null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val selectedValidator = combine(assetInfo, selectedValidatorId) { assetInfo, validatorId ->
        val assetId = assetInfo?.asset?.id ?: return@combine null
        validatorId ?: return@combine null

        stakeRepository.getStakeValidator(assetId, validatorId)
            ?: stakeRepository.getRecommended(assetId.chain).firstOrNull()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val validatorState = selectedValidator.combine(srcValidator) { selected, src ->
        selected ?: src
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val balanceContext = combine(params, assetInfo, delegation, resource) { params, assetInfo, delegation, resource ->
        BalanceRequest(params, assetInfo, delegation, resource)
    }
    .mapLatest { request ->
        val params = request.params ?: return@mapLatest null
        val assetInfo = request.assetInfo ?: return@mapLatest null
        transactionBalanceService.getContext(
            assetInfo = assetInfo,
            params = params,
            delegation = request.delegation,
            resource = request.resource,
        )
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableBalance = combine(params, assetInfo, balanceContext) { params, assetInfo, balanceContext ->
        assetInfo ?: return@combine ""
        val txType = params?.txType
        val value = txType?.let {
            Crypto(
                assetInfo.balance(
                    txType = it,
                    context = balanceContext ?: TransactionBalanceContext(),
                )
            )
        } ?: Crypto(BigInteger.ZERO)
        assetInfo.asset.format(value, 8)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    var prefillAmount = combine(
        params,
        assetInfo,
        delegation
    ) { params, assetInfo, delegation ->
        params ?: return@combine null
        assetInfo ?: return@combine null

        when (params.txType) {
            TransactionType.StakeUndelegate,
            TransactionType.StakeRedelegate,
            TransactionType.StakeWithdraw -> {
                val balance = Crypto(delegation?.base?.balance?.toBigIntegerOrNull() ?: BigInteger.ZERO)
                val value = balance.value(assetInfo.asset.decimals).stripTrailingZeros().toPlainString()
                amount = value
                value
            }
            TransactionType.StakeRewards -> {
                val balance = Crypto(delegation?.rewardsBalance() ?: BigInteger.ZERO)
                val value = balance.value(assetInfo.asset.decimals).stripTrailingZeros().toPlainString()
                amount = value
                value
            }
            else -> null
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val equivalentState = combine(
        snapshotFlow { amount },
        amountInputType,
        assetInfo
    ) { input, direction, assetInfo ->
        val priceInfo = assetInfo?.price ?: return@combine ""
        calcEquivalent(
            inputAmount = input,
            inputDirection = direction,
            asset = assetInfo.asset,
            price = priceInfo.price.price,
            currency = priceInfo.currency
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    init {
        combine(
            snapshotFlow { amount },
            amountInputType,
            assetInfo,
            params,
            combine(delegation, resource) { d, r -> d to r },
        ) { input, inputType, asset, amountParams, (delegation, resource) ->
            ValidationInputs(input, inputType, asset, amountParams, delegation, resource)
        }
        .mapLatest { validate(it) }
        .onEach { errorUIState.value = it }
        .launchIn(viewModelScope)
    }

    private suspend fun validate(inputs: ValidationInputs): AmountError {
        if (inputs.amount.isEmpty()) return AmountError.None
        if (inputs.amount.parseNumberOrNull()?.signum() == 0) return AmountError.None
        val assetInfo = inputs.assetInfo ?: return AmountError.None
        val params = inputs.params ?: return AmountError.None
        return try {
            val asset = assetInfo.asset
            val price = assetInfo.price?.price?.price ?: 0.0
            AmountValidation.validateAmount(asset, inputs.amount, getMinAmount(params.txType, asset.id.chain))
            val cryptoAmount = inputs.inputType.getAmount(inputs.amount, asset.decimals, price)
            checkBalance(assetInfo, params, inputs.delegation, inputs.resource, cryptoAmount)
            AmountError.None
        } catch (err: Throwable) {
            err as? AmountError ?: AmountError.None
        }
    }

    fun setDelegatorValidator(validatorId: String?) {
        selectedValidatorId.update { validatorId }
    }

    fun updateAmount(input: String, isMax: Boolean = false) {
        amount = input
        maxAmount.update { isMax }
    }

    fun onMaxAmount() = viewModelScope.launch {
        val assetInfo = this@AmountViewModel.assetInfo.value ?: return@launch
        val params = params.value ?: return@launch
        val txType = params.txType
        val reserveForFee = getReserveForFee(txType = txType, assetInfo.asset.chain)
        val baseBalance = transactionBalanceService.getBalance(
            assetInfo = assetInfo,
            params = params,
            delegation = delegation.value,
            resource = resource.value,
        )

        val balance = when (txType) {
            TransactionType.EarnDeposit,
            TransactionType.StakeDelegate -> if (assetInfo.stakeChain?.freezed() == true) {
                Crypto(baseBalance)
            } else {
                Crypto(maxAmountAfterReserve(baseBalance, reserveForFee))
            }
            TransactionType.StakeFreeze -> Crypto(maxAmountAfterReserve(baseBalance, reserveForFee))
            else -> Crypto(baseBalance)
        }

        updateAmount(balance.value(assetInfo.asset.decimals).stripTrailingZeros().toPlainString(), true)
    }

    fun switchInputType() {
        amountInputType.update {
            when (it) {
                AmountInputType.Crypto -> AmountInputType.Fiat
                AmountInputType.Fiat -> AmountInputType.Crypto
            }
        }
        amount = ""
    }

    fun setResource(resource: Resource) {
        this.resource.update { resource }
    }

    fun onNext(onConfirm: (ConfirmParams) -> Unit) {
        val params = params.value ?: return
        viewModelScope.launch {
            try {
                onNext(params, amount, onConfirm)
            } catch (err: Throwable) {
                when (err) {
                    is AmountError -> errorUIState.update { err }
                    else -> errorUIState.update { AmountError.Unknown(err.message ?: "Unknown error") }
                }
            }
        }
    }

    private suspend fun onNext(
        params: AmountParams,
        rawAmount: String,
        onConfirm: (ConfirmParams) -> Unit
    ) {
        val assetInfo = assetInfo.value
        val owner = assetInfo?.owner ?: return
        val validator = validatorState.value
        val delegation = delegation.value
        val asset = assetInfo.asset
        val decimals = asset.decimals
        val price = assetInfo.price?.price?.price ?: 0.0
        val destination = params.destination
        val memo = params.memo
        val inputType = amountInputType.value

        val minimumValue = getMinAmount(params.txType, asset.id.chain)
        AmountValidation.validateAmount(asset, rawAmount, minimumValue)

        val amount = inputType.getAmount(rawAmount, decimals, price)
        val balance = checkBalance(assetInfo, params, delegation, resource.value, amount)

        errorUIState.update { AmountError.None }

        val isMax = maxAmount.value || amount.atomicValue == balance
        val builder = ConfirmParams.Builder(asset, owner, amount.atomicValue, isMax)
        val nextParams = when (params.txType) {
            TransactionType.Transfer -> builder.transfer(destination!!, memo)
            TransactionType.EarnDeposit,
            TransactionType.StakeDelegate -> builder.delegate(validator ?: return)
            TransactionType.StakeUndelegate -> builder.undelegate(delegation ?: return)
            TransactionType.StakeRewards -> builder.rewards(listOfNotNull(validator))
            TransactionType.StakeRedelegate -> builder.redelegate(validator!!, delegation!!)
            TransactionType.EarnWithdraw,
            TransactionType.StakeWithdraw -> builder.withdraw(delegation!!)
            TransactionType.AssetActivation -> builder.activate()
            TransactionType.StakeFreeze -> builder.freeze(resource.value)
            TransactionType.StakeUnfreeze -> builder.unfreeze(resource.value)
            TransactionType.Swap,
            TransactionType.TransferNFT,
            TransactionType.SmartContractCall,
            TransactionType.TokenApproval,
            TransactionType.PerpetualOpenPosition,
            TransactionType.PerpetualModifyPosition,
            TransactionType.PerpetualClosePosition -> throw IllegalArgumentException()
        }
        onConfirm(nextParams)
    }

    private fun calcEquivalent(
        inputAmount: String,
        inputDirection: AmountInputType,
        asset: Asset,
        price: Double,
        currency: Currency
    ): String {
        return try {
            when (inputDirection) {
                AmountInputType.Crypto -> {
                    AmountValidation.validateAmount(asset, inputAmount, BigInteger.ZERO)
                    val amount = inputAmount.parseNumber()
                    val decimals = asset.decimals
                    val unit = Crypto(amount, decimals).convert(decimals, price)
                    currency.format(unit.atomicValue)
                }
                AmountInputType.Fiat -> {
                    val value = inputAmount.parseNumber()
                    val crypto = value.divide(price.toBigDecimal(), MathContext.DECIMAL128)
                    AmountValidation.validateAmount(asset, crypto.toString(), BigInteger.ZERO)
                    asset.format(crypto, dynamicPlace = true)
                }
            }
        } catch (_: Throwable) {
            when (inputDirection) {
                AmountInputType.Crypto -> {
                    currency.format(0.0)
                }
                AmountInputType.Fiat -> {
                    asset.format(Crypto(BigInteger.ZERO), dynamicPlace = true)
                }
            }
        }
    }

    private suspend fun checkBalance(
        assetInfo: AssetInfo,
        params: AmountParams,
        delegation: Delegation?,
        resource: Resource?,
        amount: Crypto,
    ): BigInteger {
        val balance = transactionBalanceService.getBalance(
            assetInfo = assetInfo,
            params = params,
            delegation = delegation,
            resource = resource,
        )
        val availableBalance = balance.toBigDecimal().movePointLeft(assetInfo.asset.decimals)
        AmountValidation.validateBalance(assetInfo, amount, availableBalance)
        return balance
    }

    private fun getMinAmount(txType: TransactionType, chain: Chain): BigInteger {
        return when (txType) {
            TransactionType.StakeRedelegate,
            TransactionType.StakeFreeze,
            TransactionType.StakeDelegate -> BigInteger.valueOf(
                Config().getStakeConfig(chain.string).minAmount.toLong()
            )
            else -> BigInteger.ZERO
        }
    }

    companion object {
        fun maxAmountAfterReserve(balance: BigInteger, reserve: BigInteger): BigInteger =
            maxOf(balance - reserve, BigInteger.ZERO)
    }

    private fun getReserveForFee(txType: TransactionType, chain: Chain) = when (txType) {
        TransactionType.StakeFreeze -> BigInteger.valueOf(Config().getStakeConfig(chain.string).reservedForFees.toLong())
        TransactionType.StakeDelegate -> when (chain) {
            Chain.Tron -> BigInteger.ZERO
            else -> BigInteger.valueOf(Config().getStakeConfig(chain.string).reservedForFees.toLong())
        }
        else -> BigInteger.ZERO
    }
}

private data class BalanceRequest(
    val params: AmountParams?,
    val assetInfo: AssetInfo?,
    val delegation: Delegation?,
    val resource: Resource,
)

private data class ValidationInputs(
    val amount: String,
    val inputType: AmountInputType,
    val assetInfo: AssetInfo?,
    val params: AmountParams?,
    val delegation: Delegation?,
    val resource: Resource,
)
