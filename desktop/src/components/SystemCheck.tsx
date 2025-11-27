import { SystemProfile, Recommendation } from "../lib/api";

interface Props {
  profile: SystemProfile | null;
  recommendation: Recommendation | null;
  onContinue: () => void;
}

export function SystemCheck({ profile, recommendation, onContinue }: Props) {
  return (
    <>
      <div className="card">
        <h3>System Profile</h3>
        {profile ? (
          <ul>
            <li>OS: {profile.os_name}</li>
            <li>RAM: {profile.total_memory_gb} GB</li>
            <li>CPU Cores: {profile.cpu_cores}</li>
            <li>Arch: {profile.architecture}</li>
          </ul>
        ) : (
          <p>Scanning system...</p>
        )}
      </div>

      {recommendation && (
        <div className="card highlight">
          <h3>Recommendation: {recommendation.recommended_engine}</h3>
          <p>{recommendation.reason}</p>
        </div>
      )}

      <div className="actions">
        <button onClick={onContinue}>Continue</button>
      </div>
    </>
  );
}

