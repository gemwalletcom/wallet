// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemConfirmData
public import struct Gemstone.GemConfirmInput
public import struct Gemstone.GemConfirmLoadOptions
public import protocol Gemstone.GemConfirmServiceProtocol
public import enum Gemstone.GemExecuteResult
public import struct Gemstone.GemSendInput
public import protocol Gemstone.GemTransactionSigner
import Foundation

public final class GemConfirmServiceMock: GemConfirmServiceProtocol, @unchecked Sendable {
    private let executeResult: Result<GemExecuteResult, any Error>
    private let lock = NSLock()
    private var inputs: [GemSendInput] = []

    public var executedInputs: [GemSendInput] { lock.withLock { inputs } }

    public init(execute: Result<GemExecuteResult, any Error> = .success(.sent(hashes: [], transactions: []))) {
        executeResult = execute
    }

    public func load(input _: GemConfirmInput, options _: GemConfirmLoadOptions) async throws -> GemConfirmData {
        fatalError("not used")
    }

    public func execute(input: GemSendInput, signer _: any GemTransactionSigner) async throws -> GemExecuteResult {
        lock.withLock { inputs.append(input) }
        return try executeResult.get()
    }
}
