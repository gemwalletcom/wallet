import Foundation
import class Gemstone.GemMnemonic
@testable import Keystore
import KeystoreTestKit
import Primitives
import Testing

struct LocalKeystoreTests {
    @Test
    func testImportWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let words = try GemMnemonic().generate(wordCount: 12)
            let wallet = try keystore.importWallet(
                name: "test",
                type: .phrase(words: words, chains: [.ethereum]),
            )

            #expect(wallet.accounts.count == 1)
            #expect(wallet.accounts.first?.chain == .ethereum)
        }
    }

    @Test
    func importSolanaWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let wallet = try keystore.importWallet(
                name: "Solana Wallet",
                type: .phrase(words: LocalKeystore.words, chains: [.solana]),
            )

            #expect(wallet.accounts.count == 1)
            #expect(wallet.accounts.first?.chain == .solana)
            #expect(wallet.accounts.first?.address == "57mwmnV2rFuVDmhiJEjonD7cfuFtcaP9QvYNGfDEWK71")
        }
    }

    @Test
    func importEthereumWallet() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let chains: [Chain] = [.ethereum, .smartChain, .blast]

            let wallet = try keystore.importWallet(
                name: "test",
                type: .phrase(words: LocalKeystore.words, chains: chains),
            )

            #expect(wallet.accounts == chains.map {
                Account(chain: $0,
                        address: "0x8f348F300873Fd5DA36950B2aC75a26584584feE",
                        derivationPath: "m/44'/60'/0'/0/0",
                        extendedPublicKey: "045a0c6b83b8bd9827e507270cadb499b7e3a9095246f6a2213281f783d877c98b256742741b0639f317768fe4f4c2762660c2112283a7685d815507dee3229173")
            })
        }
    }

    @Test
    func exportSolanaPrivateKey() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let hex = "0xb9095df5360714a69bc86ca92f6191e60355f206909982a8409f7b8358cf41b0"
            let wallet = try keystore.importWallet(
                name: "Test Solana",
                type: .privateKey(text: hex, chain: .solana),
            )

            let exported = try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: .solana)
            #expect(exported == "DTJi5pMtSKZHdkLX4wxwvjGjf2xwXx1LSuuUZhugYWDV")

            let keystore2 = LocalKeystore.mock()
            let wallet2 = try keystore2.importWallet(
                name: "Test Solana 2",
                type: .privateKey(text: exported, chain: .solana),
            )
            let exportedKey = try await keystore2.getPrivateKeyEncoded(wallet: wallet2, chain: .solana)
            #expect(exportedKey == exported)
        }
    }

    @Test
    func exportEthereumPrivateKey() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let hex = "0x30df0ffc2b43717f4653c2a1e827e9dfb3d9364e019cc60092496cd4997d5d6e"
            let wallet = try keystore.importWallet(
                name: "Test Ethereum",
                type: .privateKey(text: hex, chain: .ethereum),
            )

            let exported = try await keystore.getPrivateKeyEncoded(wallet: wallet, chain: .ethereum)
            #expect(exported == hex)
        }
    }

    @Test
    func deriveAddress() async {
        await #expect(throws: Never.self) {
            let keystore = LocalKeystore.mock()
            let chains = AssetConfiguration.allChains
            let wallet = try keystore.importWallet(
                name: "test",
                type: .phrase(words: LocalKeystore.words, chains: chains),
            )

            #expect(wallet.accounts.count == chains.count)

            for account in wallet.accounts {
                let chain = account.chain
                let derivedAddress = account.address
                let expected = switch chain {
                case .bitcoin:
                    "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d"
                case .litecoin:
                    "ltc1qhd8fxxp2dx3vsmpac43z6ev0kllm4n53t5sk0u"
                case .ethereum,
                     .smartChain,
                     .polygon,
                     .arbitrum,
                     .optimism,
                     .base,
                     .avalancheC,
                     .opBNB,
                     .fantom,
                     .gnosis,
                     .manta,
                     .blast,
                     .zkSync,
                     .linea,
                     .mantle,
                     .celo,
                     .world,
                     .sonic,
                     .seiEvm,
                     .abstract,
                     .berachain,
                     .ink,
                     .unichain,
                     .hyperliquid,
                     .monad,
                     .hyperCore,
                     .plasma,
                     .xLayer,
                     .robinhood,
                     .stable,
                     .tempo:
                    "0x8f348F300873Fd5DA36950B2aC75a26584584feE"
                case .solana:
                    "57mwmnV2rFuVDmhiJEjonD7cfuFtcaP9QvYNGfDEWK71"
                case .thorchain:
                    "thor1c8jd7ad9pcw4k3wkuqlkz4auv95mldr2kyhc65"
                case .mayachain:
                    "maya1c8jd7ad9pcw4k3wkuqlkz4auv95mldr2knf5vy"
                case .cosmos:
                    "cosmos142j9u5eaduzd7faumygud6ruhdwme98qsy2ekn"
                case .osmosis:
                    "osmo142j9u5eaduzd7faumygud6ruhdwme98qclefqp"
                case .ton:
                    "UQDgEMqToTacHic7SnvnPFmvceG5auFkCcAw0mSCvzvKUaT4"
                case .tron:
                    "TQ5NMqJjhpQGK7YJbESKtNCo86PJ89ujio"
                case .doge:
                    "DJRFZNg8jkUtjcpo2zJd92FUAzwRjitw6f"
                case .aptos:
                    "0x07968dab936c1bad187c60ce4082f307d030d780e91e694ae03aef16aba73f30"
                case .sui:
                    "0xada112cfb90b44ba889cc5d39ac2bf46281e4a91f7919c693bcd9b8323e81ed2"
                case .xrp:
                    "rPwE3gChNKtZ1mhH3Ko8YFGqKmGRWLWXV3"
                case .celestia:
                    "celestia142j9u5eaduzd7faumygud6ruhdwme98qpwmfv7"
                case .injective:
                    "inj13u6g7vqgw074mgmf2ze2cadzvkz9snlwcrtq8a"
                case .sei:
                    "sei142j9u5eaduzd7faumygud6ruhdwme98qagm0sj"
                case .noble:
                    "noble142j9u5eaduzd7faumygud6ruhdwme98qc8l3wa"
                case .near:
                    "0c91f6106ff835c0195d5388565a2d69e25038a7e23d26198f85caf6594117ec"
                case .stellar:
                    "GA3H6I4C5XUBYGVB66KXR27JV5KS3APSTKRUWOIXZ5MVWZKVTLXWKZ2P"
                case .bitcoinCash:
                    "qpzl3jxkzgvfd9flnd26leud5duv795fnv7vuaha70"
                case .algorand:
                    "JTJWO524JXIHVPGBDWFLJE7XUIA32ECOZOBLF2QP3V5TQBT3NKZSCG67BQ"
                case .polkadot:
                    "13nN6BGAoJwd7Nw1XxeBCx5YcBXuYnL94Mh7i3xBprqVSsFk"
                case .cardano:
                    "addr1qyr8jjfnypp95eq74aqzn7ss687ehxclgj7mu6gratmg3mul2040vt35dypp042awzsjk5xm3zr3zm5qh7454uwdv08s84ray2"
                case .zcash:
                    "t1YYnByMzdGhQv3W3rnjHMrJs6HH4Y231gy"
                }

                #expect(derivedAddress == expected, "\(chain) failed to match address")
            }
        }
    }
}
