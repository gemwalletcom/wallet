package com.gemwallet.android.blockchain.clients.solana

import com.gemwallet.android.blockchain.includeLibs
import com.gemwallet.android.blockchain.services.SignService
import com.gemwallet.android.ext.asset
import com.gemwallet.android.math.append0x
import com.gemwallet.android.math.hex
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.model.Fee
import com.gemwallet.android.testkit.TEST_PHRASE
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.SolanaTokenProgramId
import junit.framework.TestCase.assertEquals
import kotlinx.coroutines.runBlocking
import org.junit.Test
import wallet.core.jni.Base58
import wallet.core.jni.CoinType
import wallet.core.jni.Curve
import wallet.core.jni.HDWallet
import wallet.core.jni.PrivateKey
import java.math.BigInteger

class TestSolanaSigner {
    companion object {
        init {
            includeLibs()
        }
    }

    val privateKey = HDWallet(TEST_PHRASE, "").getKeyForCoin(CoinType.SOLANA).data()
    val senderAddress = "5yUxrLd6C5nSDzpvg9bNpQMXpcw6J6pSVrYQmr6Bmyp6"

    @Test
    fun testSolana_sign_message() {
        val message = "hello".toByteArray()
        val expected = Base58.encodeNoCheck(PrivateKey(privateKey).sign(message, Curve.ED25519))

        val result = runBlocking {
            SignService().signMessage(Chain.Solana, message, privateKey)
        }

        assertEquals(expected, result.decodeToString())
    }

    @Test
    fun testSolana_native_transfer() {
        val signer = SignService()
        val params = ConfirmParams.Builder(
            asset = Chain.Solana.asset(),
            from = Account(Chain.Solana, senderAddress, ""),
            amount = BigInteger.valueOf(10_000_000)
        )
            .transfer(destination = DestinationAddress("4Yu2e1Wz5T1Ci2hAPswDqvMgSnJ1Ftw7ZZh8x7xKLx7S")) as ConfirmParams.TransferParams.Native
        val chainData = SolanaChainData(
            blockHash = "kiEPF6aKvEsj5nbi4FBvgRRm9ha36Y3cgDU9qnUKt32",
            recipientTokenAddress = null,
            senderTokenAddress = "",
            tokenProgram = SolanaTokenProgramId.Token
        )
        val result = runBlocking {
            signer.signNativeTransfer(
                params,
                chainData,
                BigInteger.ZERO,
                Fee.Solana(
                    amount = BigInteger("105005000"),
                    minerFee = BigInteger("1050000000"),
                    maxGasPrice = BigInteger("5000"),
                    unitFee = BigInteger("5000"),
                    limit = BigInteger("100000"),
                    feeAssetId = AssetId(Chain.Solana),
                    priority = FeePriority.Normal,
                    options = emptyMap(),
                ),
                privateKey
            )
        }
        assertEquals(
            "0x415965576d664f4538764f357870302b7159557733437a4450713839724b4c587a7976546a587673" +
                    "7a37796f42746f44324f30394f3148554e78584f704c456a43645371397876414a4f6c41564a486b" +
                    "397568707a516342414149455365626b44466a2b415242396b4b486b394f4167745057456e614370" +
                    "426a475a3869707a61372f4e43577330767554324f647746546758414767305a6175317474703157" +
                    "354f7153437a77434c53384e5738357a71514d47526d2f6c495263792f2b7974756e4c446d2b6538" +
                    "6a4f573778666353617978446d7a7041414141414141414141414141414141414141414141414141" +
                    "41414141414141414141414141414141414141414141414c4d706730536d427863684e486e756872" +
                    "566468464259677863634c722b5370616932436959444a4b51514d4341416b4469424d4141414141" +
                    "41414143414155436f49594241414d434141454d41674141414141414141414141414141", result.first().hex.append0x()
        )
    }

    @Test
    fun testSolana_token_transfer() {
        val signer = SignService()
        val params = ConfirmParams.Builder(
            asset = Asset(
                AssetId(Chain.Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),
                "",
                "",
                8,
                AssetType.SPL
            ),
            from = Account(Chain.Solana, senderAddress, ""),
            amount = BigInteger.valueOf(10_000_000)
        )
            .transfer(destination = DestinationAddress("AGkXQZ9qm99xukisDUHvspWHESrcjs8Y4AmQQgef3BRh")) as ConfirmParams.TransferParams.Token
        val chainData = SolanaChainData(
            blockHash = "kiEPF6aKvEsj5nbi4FBvgRRm9ha36Y3cgDU9qnUKt32",
            recipientTokenAddress = "DVWPV7brSbPDkA7a3qdn6UJsVc3J3DyhQhjNaZeZqwzo",
            senderTokenAddress = "DVWPV7brSbPDkA7a3qdn6UJsVc3J3DyhQhjNaZeZqwzo",
            tokenProgram = SolanaTokenProgramId.Token
        )
        val result = runBlocking {
            signer.signTokenTransfer(
                params,
                chainData,
                BigInteger.ZERO,
                Fee.Solana(
                    amount = BigInteger("105005000"),
                    minerFee = BigInteger("1050000000"),
                    maxGasPrice = BigInteger("5000"),
                    unitFee = BigInteger("5000"),
                    limit = BigInteger("100000"),
                    feeAssetId = AssetId(Chain.Solana),
                    priority = FeePriority.Normal,
                    options = emptyMap(),
                ),
                privateKey
            )
        }
        assertEquals(
            "0x4163456263336c444271665a76794f3954784d4e39647244584c6c374673626e664c4f453031362f" +
                    "4c5a374a737059497248744a4579335a536354715842396143317362413734444e5033735a79686b" +
                    "6c354f347967304241414d465365626b44466a2b415242396b4b486b394f4167745057456e614370" +
                    "426a475a3869707a61372f4e435775356d6277492b744e3454586473757149654b725254647a734b" +
                    "644444707a385843635179334a766a50304d3442446d43763762496e4637316a4753395546466f2f" +
                    "6c6c6f7a75344c5378774b657373346549494a6b41775a47622b5568467a4c2f374b323663734f62" +
                    "3537794d3562764639784a724c454f624f6b41414141414733666268313257686b396e4c3455624f" +
                    "36336d73484c5346375639624e3545366a50574666763841715173796d44524b5948467945306565" +
                    "364774563245554669444678777576354b6c714c594b4a674d6b704241774d4143514f4945774141" +
                    "4141414141414d4142514b676867454142415142416745414367774141414141414141414141673d", result.first().hex.append0x()
        )
    }

    @Test
    fun testSolana_token2022_transfer() {
        val signer = SignService()
        val params = ConfirmParams.Builder(
            asset = Asset(
                AssetId(Chain.Solana, "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo"),
                "",
                "",
                8,
                AssetType.SPL2022
            ),
            from = Account(Chain.Solana, senderAddress, ""),
            amount = BigInteger.valueOf(10_000_000)
        )
            .transfer(destination = DestinationAddress("AGkXQZ9qm99xukisDUHvspWHESrcjs8Y4AmQQgef3BRh")) as ConfirmParams.TransferParams.Token
        val chainData = SolanaChainData(
            blockHash = "kiEPF6aKvEsj5nbi4FBvgRRm9ha36Y3cgDU9qnUKt32",
            recipientTokenAddress = "87vTugUvkkepa84mBRfENnvkPQRj5EZSkiG8XyFAhbQQ",
            senderTokenAddress = "87vTugUvkkepa84mBRfENnvkPQRj5EZSkiG8XyFAhbQQ",
            tokenProgram = SolanaTokenProgramId.Token2022
        )
        val result = runBlocking {
            signer.signTokenTransfer(
                params,
                chainData,
                BigInteger.ZERO,
                Fee.Solana(
                    amount = BigInteger("105005000"),
                    minerFee = BigInteger("1050000000"),
                    maxGasPrice = BigInteger("5000"),
                    unitFee = BigInteger("5000"),
                    limit = BigInteger("100000"),
                    feeAssetId = AssetId(Chain.Solana),
                    priority = FeePriority.Normal,
                    options = emptyMap(),
                ),
                privateKey
            )
        }
        assertEquals(
            "0x416642774d397335796f63362b3845727a325432664e595a35747a35466f41786f69485463664d6f" +
                    "744a4967757549733931386f507673584c59366964776e5a6f41696b506d6e356f4a775865443153" +
                    "3171354a6c51514241414d465365626b44466a2b415242396b4b486b394f4167745057456e614370" +
                    "426a475a3869707a61372f4e43577470783737467363375a7773776d4234324a6539304538487a62" +
                    "2b59486131534a74465451777a3853517052655353447473696971487430636467552b566b666b35" +
                    "5849514b6e4f505a394e57366654704c696e536541775a47622b5568467a4c2f374b323663734f62" +
                    "3537794d3562764639784a724c454f624f6b41414141414733666268376e57503368684358627a6b" +
                    "624d33617468723854594f354453662b76666b6f324b474c2f4173796d44524b5948467945306565" +
                    "364774563245554669444678777576354b6c714c594b4a674d6b704241774d4143514f4945774141" +
                    "4141414141414d4142514b676867454142415142416745414367774141414141414141414141673d", result.first().hex.append0x()
        )
    }

}
