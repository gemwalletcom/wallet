// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Signer
import Testing

struct SolanaSignerTests {
    let fee = Fee(fee: .zero, gasPriceType: .solana(gasPrice: 5000, priorityFee: 10000, unitPrice: 200), gasLimit: 125_000)
    let senderAddress = "FEG8HUjcdTScQ27B7Hay2Yqzdy9UuPWGBZoCfkyFb5wf"
    let signer = ChainSigner(chain: .solana)

    @Test
    func testTransfer() throws {
        let asset = Asset(.solana).chain.asset
        let input = SignerInput(
            type: .transfer(asset),
            asset: asset,
            value: .zero,
            fee: fee,
            isMaxAmount: false,
            memo: .none,
            senderAddress: senderAddress,
            destinationAddress: "HVoJWyPbQn4XikG9BY2A8wP27HJQzHAoDnAs1SfsATes",
            metadata: .solana(
                senderTokenAddress: nil,
                recipientTokenAddress: nil,
                tokenProgram: nil,
                nft: nil,
                blockHash: "8ntZRPm8pbf4R4pTWsVnTdgqXA35yYXSz8TxUzwBhXEK",
            ),
        )

        let result = try signer.signTransfer(input: input, privateKey: TestPrivateKey)
        #expect(result == "AQ+bcpkOGB15GeVJxnh3F9oQmLUf98OkLMonJusdLm85R2ukxnqgd6OmmP1XgBUL7GbN4t2jRJsOQOGiQkEcVwsBAAIE02lFIZfCpWSB5eLT6L8D3iNJ9npjFRlWgiIIwjNK3uL1G5t56F3oMXO9/md9Dan95RdKnFZ/h5iqL/+hVtYYxAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABzwz4o6+Cji9oIdB7FElRcPSFxAzYV8cPxQk26SYaknAMCAAkDyAAAAAAAAAACAAUCSOgBAAMCAAEMAgAAAAAAAAAAAAAA")
    }

    @Test
    func tokenTransfer() throws {
        let asset = Asset.mock(id: AssetId(chain: .solana, tokenId: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), decimals: 8, type: .spl)
        let input = SignerInput(
            type: .transfer(asset),
            asset: asset,
            value: .zero,
            fee: fee,
            isMaxAmount: false,
            memo: .none,
            senderAddress: senderAddress,
            destinationAddress: "HVoJWyPbQn4XikG9BY2A8wP27HJQzHAoDnAs1SfsATes",
            metadata: .solana(
                senderTokenAddress: "DVWPV7brSbPDkA7a3qdn6UJsVc3J3DyhQhjNaZeZqwzo",
                recipientTokenAddress: "8ntZRPm8pbf4R4pTWsVnTdgqXA35yYXSz8TxUzwBhXEK",
                tokenProgram: .token,
                nft: nil,
                blockHash: "8ntZRPm8pbf4R4pTWsVnTdgqXA35yYXSz8TxUzwBhXEK",
            ),
        )

        let result = try signer.signTokenTransfer(input: input, privateKey: TestPrivateKey)
        #expect(result == "AZDsKWD23xJAiSBqWBaypGgknAVaIWw36s3PZdgaZiVhZn2zFJDbSms2WVU8hhbiQhDN6VYz0itrakUYbYnc3AsBAAMG02lFIZfCpWSB5eLT6L8D3iNJ9npjFRlWgiIIwjNK3uK5mbwI+tN4TXdsuqIeKrRTdzsKdDDpz8XCcQy3JvjP0HPDPijr4KOL2gh0HsUSVFw9IXEDNhXxw/FCTbpJhqSczgEOYK/tsicXvWMZL1QUWj+WWjO7gtLHAp6yzh4ggmQDBkZv5SEXMv/srbpyw5vnvIzlu8X3EmssQ5s6QAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpc8M+KOvgo4vaCHQexRJUXD0hcQM2FfHD8UJNukmGpJwDBAAJA8gAAAAAAAAABAAFAkjoAQAFBAEDAgAKDAAAAAAAAAAACA==")
    }

    @Test
    func tokenTransferNewAccount() throws {
        let asset = Asset.mock(id: AssetId(chain: .solana, tokenId: "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"), decimals: 8, type: .spl)
        let input = SignerInput(
            type: .transfer(asset),
            asset: asset,
            value: .zero,
            fee: fee,
            isMaxAmount: false,
            memo: .none,
            senderAddress: senderAddress,
            destinationAddress: "HVoJWyPbQn4XikG9BY2A8wP27HJQzHAoDnAs1SfsATes",
            metadata: .solana(
                senderTokenAddress: "DVWPV7brSbPDkA7a3qdn6UJsVc3J3DyhQhjNaZeZqwzo",
                recipientTokenAddress: nil,
                tokenProgram: .token,
                nft: nil,
                blockHash: "8ntZRPm8pbf4R4pTWsVnTdgqXA35yYXSz8TxUzwBhXEK",
            ),
        )

        let result = try signer.signTokenTransfer(input: input, privateKey: TestPrivateKey)
        #expect(result == "AbuNML07oatutMK8MiMw9fHiFtymJ51ZEmgp5A9dQt1ESrR8L7Rckn0+CM7Tp7MD4d95gBlixCt/KsrCIayflwoBAAcK02lFIZfCpWSB5eLT6L8D3iNJ9npjFRlWgiIIwjNK3uLwhdgOizjjG7Mj811OLchuXyuo+2ii49Swx83kAelI9bmZvAj603hNd2y6oh4qtFN3Owp0MOnPxcJxDLcm+M/Q9Rubeehd6DFzvf5nfQ2p/eUXSpxWf4eYqi//oVbWGMTOAQ5gr+2yJxe9YxkvVBRaP5ZaM7uC0scCnrLOHiCCZAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABt324ddloZPZy+FGzut5rBy0he1fWzeROoz1hX7/AKkGp9UXGSxcUSGMyUw9SvF/WNruCJuh/UTj29mKAAAAAAMGRm/lIRcy/+ytunLDm+e8jOW7xfcSayxDmzpAAAAAjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+Flzwz4o6+Cji9oIdB7FElRcPSFxAzYV8cPxQk26SYaknAQIAAkDyAAAAAAAAAAIAAUCSOgBAAkHAAEDBAUGBwEBBgQCBAEACgwAAAAAAAAAAAg=")
    }

    @Test
    func signSolanaMessage() throws {
        let privateKey = try #require(Data(hexString: "0CF3BCE421F870B1D54B27632AED05B7BC2AAF4AA818ED1275FB3A75518E60CB"))
        let message = "hello world"
        let dataMessage = try #require(message.data(using: .utf8))

        let signature = try signer.signMessage(message: .raw(dataMessage), privateKey: privateKey)

        #expect(signature == "2gK63KVgpUMjT612P2iyL1TCZx5zmwbXjNMQ9PqkVrLsUpNuPWUhJhGLp4puzXu87AoNtMASkzziUJmkKCv3wESR")
    }
}
