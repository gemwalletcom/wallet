import Foundation

public extension Data {
    func encodeString() throws -> String {
        guard let string = String(data: self, encoding: .utf8) else {
            throw AnyError("unable to encode string")
        }
        return string
    }

    var hex: String {
        map { String(format: "%02x", $0) }.joined()
    }

    mutating func zeroize() {
        guard !isEmpty else { return }
        withUnsafeMutableBytes { bytes in
            guard let baseAddress = bytes.baseAddress else { return }
            memset_s(baseAddress, bytes.count, 0, bytes.count)
        }
    }
}
