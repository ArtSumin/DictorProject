import Foundation
import AVFoundation

public protocol AudioRecordingService: Sendable {
    func requestPermission() async -> Bool
    func startRecording() async throws
    func stopRecording() async throws -> URL
    /// Returns dBFS (-160…0). Use -160 when not recording.
    func currentLevelDecibels() async -> Float
}

public actor AVAudioRecorderService: AudioRecordingService {
    private var audioRecorder: AVAudioRecorder?
    private var recordingURL: URL?

    public init() {}

    public func requestPermission() async -> Bool {
        if #available(macOS 14.0, *) {
            return await AVCaptureDevice.requestAccess(for: .audio)
        } else {
            return await withCheckedContinuation { continuation in
                AVCaptureDevice.requestAccess(for: .audio) { granted in
                    continuation.resume(returning: granted)
                }
            }
        }
    }

    public func startRecording() async throws {
        if audioRecorder?.isRecording == true {
            audioRecorder?.stop()
        }
        audioRecorder = nil

        let tempDir = FileManager.default.temporaryDirectory
        let fileName = "dictor_recording_\(UUID().uuidString).wav"
        let url = tempDir.appendingPathComponent(fileName)
        self.recordingURL = url

        let settings: [String: Any] = [
            AVFormatIDKey: Int(kAudioFormatLinearPCM),
            AVSampleRateKey: 16000.0,
            AVNumberOfChannelsKey: 1,
            AVLinearPCMBitDepthKey: 16,
            AVLinearPCMIsFloatKey: false,
            AVLinearPCMIsBigEndianKey: false
        ]

        audioRecorder = try AVAudioRecorder(url: url, settings: settings)
        audioRecorder?.isMeteringEnabled = true
        audioRecorder?.prepareToRecord()
        audioRecorder?.record()
    }

    public func currentLevelDecibels() async -> Float {
        guard let recorder = audioRecorder, recorder.isRecording else {
            return -160
        }
        recorder.updateMeters()
        return recorder.averagePower(forChannel: 0)
    }

    public func stopRecording() async throws -> URL {
        audioRecorder?.stop()
        audioRecorder = nil

        guard let url = recordingURL else {
            throw NSError(domain: "AudioRecording", code: 0, userInfo: [NSLocalizedDescriptionKey: "No recording URL found"])
        }
        return url
    }
}
