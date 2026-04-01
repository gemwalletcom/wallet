import Foundation
import Primitives

public enum SignMessage {
    case typed(String)
    case raw(Data)
}

public protocol Signable {
    func signData(input: SignerInput, privateKey: Data) throws -> String
    func signTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signTokenTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signNftTransfer(input: SignerInput, privateKey: Data) throws -> String
    func signSwap(input: SignerInput, privateKey: Data) throws -> [String]
    func signTokenApproval(input: SignerInput, privateKey: Data) throws -> String
    func signStake(input: SignerInput, privateKey: Data) throws -> [String]
    func signEarn(input: SignerInput, privateKey: Data) throws -> [String]
    func signMessage(message: SignMessage, privateKey: Data) throws -> String
    func signAccountAction(input: SignerInput, privateKey: Data) throws -> String
    func signPerpetual(input: SignerInput, privateKey: Data) throws -> [String]
    func signWithdrawal(input: SignerInput, privateKey: Data) throws -> String
}

public extension Signable {
    func signTokenTransfer(input _: SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signTokenTransfer method")
    }

    func signNftTransfer(input _: SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signNftTransfer method")
    }

    func signAccountAction(input _: SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signAccountAction method")
    }

    func signData(input _: Primitives.SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signData method")
    }

    func signSwap(input _: SignerInput, privateKey _: Data) throws -> [String] {
        throw AnyError("unimplemented: signSwap method")
    }

    func signTokenApproval(input _: SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signTokenApproval method")
    }

    func signStake(input _: SignerInput, privateKey _: Data) throws -> [String] {
        throw AnyError("unimplemented: signStake method")
    }

    func signEarn(input _: SignerInput, privateKey _: Data) throws -> [String] {
        throw AnyError("unimplemented: signEarn method")
    }

    func signMessage(message _: SignMessage, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signMessage method")
    }

    func signPerpetual(input _: SignerInput, privateKey _: Data) throws -> [String] {
        throw AnyError("unimplemented: signPerpetual method")
    }

    func signWithdrawal(input _: SignerInput, privateKey _: Data) throws -> String {
        throw AnyError("unimplemented: signWithdrawal method")
    }
}
