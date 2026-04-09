package com.gemwallet.android.blockchain.operators

import com.gemwallet.android.blockchain.includeLibs
import com.gemwallet.android.blockchain.operators.walletcore.WCCreateAccountOperator
import com.gemwallet.android.blockchain.testPhrase
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import junit.framework.TestCase.assertEquals
import org.junit.Test

class TestCreateAccountOperator {
    companion object {
        private const val iosLocalKeystorePhrase =
            "shoot island position soft burden budget tooth cruel issue economy destroy above"

        init {
            includeLibs()
        }
    }

    @Test
    fun testCreate_account_solana() {
        val operator = WCCreateAccountOperator()
        val result = operator(walletType = WalletType.Multicoin, data = testPhrase, Chain.Solana)
        assertEquals("4Yu2e1Wz5T1Ci2hAPswDqvMgSnJ1Ftw7ZZh8x7xKLx7S", result.address)
        assertEquals("m/44'/501'/0'", result.derivationPath)
        assertEquals("", result.extendedPublicKey)
    }

    @Test
    fun testCreate_account_bitcoincache() {
        val operator = WCCreateAccountOperator()
        val result = operator(walletType = WalletType.Multicoin, data = testPhrase, Chain.BitcoinCash)
        assertEquals("qq29xrkkd68alnrca375qlfyhwdqdkevsvmgkq9cmw", result.address)
        assertEquals("m/44'/145'/0'/0/0", result.derivationPath)
        assertEquals("xpub6Cd3LU6iyrbbhxPRYZpE5hGUdmrQVpQ79i9RYNLrs2iVrtYkKRv6swMWeTpPfomebgisrRGPrFvt1qaFiZLLuQdSFRVBWdbKD4HWnMrFsjR", result.extendedPublicKey)
    }


    @Test
    fun testCreate_account_evm() {
        val operator = WCCreateAccountOperator()
        val result = operator(walletType = WalletType.Multicoin, data = testPhrase, Chain.Ethereum)
        assertEquals("0x9b1DB81180c31B1b428572Be105E209b5A6222b7", result.address)
        assertEquals("m/44'/60'/0'/0/0", result.derivationPath)
        assertEquals("", result.extendedPublicKey)
    }

    @Test
    fun testCreate_account_derive_address_matches_ios_local_keystore() {
        val operator = WCCreateAccountOperator()

        Chain.entries.forEach { chain ->
            val account = operator(
                walletType = WalletType.Multicoin,
                data = iosLocalKeystorePhrase,
                chain = chain,
            )

            assertEquals("Unexpected derived address for $chain", expectedAddress(chain), account.address)
        }
    }

    private fun expectedAddress(chain: Chain): String = when (chain) {
        Chain.Bitcoin -> "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d"
        Chain.BitcoinCash -> "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70"
        Chain.Litecoin -> "ltc1qhd8fxxp2dx3vsmpac43z6ev0kllm4n53t5sk0u"
        Chain.Ethereum,
        Chain.SmartChain,
        Chain.Polygon,
        Chain.Arbitrum,
        Chain.Optimism,
        Chain.Base,
        Chain.AvalancheC,
        Chain.OpBNB,
        Chain.Fantom,
        Chain.Gnosis,
        Chain.Manta,
        Chain.Blast,
        Chain.ZkSync,
        Chain.Linea,
        Chain.Mantle,
        Chain.Celo,
        Chain.World,
        Chain.Sonic,
        Chain.SeiEvm,
        Chain.Abstract,
        Chain.Berachain,
        Chain.Ink,
        Chain.Unichain,
        Chain.Hyperliquid,
        Chain.HyperCore,
        Chain.Monad,
        Chain.Plasma,
        Chain.XLayer,
        Chain.Stable -> "0x8f348F300873Fd5DA36950B2aC75a26584584feE"
        Chain.Solana -> "57mwmnV2rFuVDmhiJEjonD7cfuFtcaP9QvYNGfDEWK71"
        Chain.Thorchain -> "thor1c8jd7ad9pcw4k3wkuqlkz4auv95mldr2kyhc65"
        Chain.Cosmos -> "cosmos142j9u5eaduzd7faumygud6ruhdwme98qsy2ekn"
        Chain.Osmosis -> "osmo142j9u5eaduzd7faumygud6ruhdwme98qclefqp"
        Chain.Ton -> "UQDgEMqToTacHic7SnvnPFmvceG5auFkCcAw0mSCvzvKUaT4"
        Chain.Tron -> "TQ5NMqJjhpQGK7YJbESKtNCo86PJ89ujio"
        Chain.Doge -> "DJRFZNg8jkUtjcpo2zJd92FUAzwRjitw6f"
        Chain.Zcash -> "t1YYnByMzdGhQv3W3rnjHMrJs6HH4Y231gy"
        Chain.Aptos -> "0x7968dab936c1bad187c60ce4082f307d030d780e91e694ae03aef16aba73f30"
        Chain.Sui -> "0xada112cfb90b44ba889cc5d39ac2bf46281e4a91f7919c693bcd9b8323e81ed2"
        Chain.Xrp -> "rPwE3gChNKtZ1mhH3Ko8YFGqKmGRWLWXV3"
        Chain.Celestia -> "celestia142j9u5eaduzd7faumygud6ruhdwme98qpwmfv7"
        Chain.Injective -> "inj13u6g7vqgw074mgmf2ze2cadzvkz9snlwcrtq8a"
        Chain.Sei -> "sei142j9u5eaduzd7faumygud6ruhdwme98qagm0sj"
        Chain.Noble -> "noble142j9u5eaduzd7faumygud6ruhdwme98qc8l3wa"
        Chain.Near -> "0c91f6106ff835c0195d5388565a2d69e25038a7e23d26198f85caf6594117ec"
        Chain.Stellar -> "GA3H6I4C5XUBYGVB66KXR27JV5KS3APSTKRUWOIXZ5MVWZKVTLXWKZ2P"
        Chain.Algorand -> "JTJWO524JXIHVPGBDWFLJE7XUIA32ECOZOBLF2QP3V5TQBT3NKZSCG67BQ"
        Chain.Polkadot -> "13nN6BGAoJwd7Nw1XxeBCx5YcBXuYnL94Mh7i3xBprqVSsFk"
        Chain.Cardano -> "addr1qyr8jjfnypp95eq74aqzn7ss687ehxclgj7mu6gratmg3mul2040vt35dypp042awzsjk5xm3zr3zm5qh7454uwdv08s84ray2"
    }
}
