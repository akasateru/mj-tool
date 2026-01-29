export type WinMethod = "tsumo" | "ron";
export type WinnerRole = "dealer" | "non_dealer";
export type Discarder = "opponent1" | "opponent2" | "opponent3";

export type Fact = {
  winner_role: WinnerRole;
  win_method: WinMethod;
  han: number;
  fu: number;
  honba: number;
  riichi_sticks: number;
  discarder?: Discarder | null;
};

export type ValidationError =
  | { type: "han_must_be_positive"; detail: string }
  | { type: "fu_invalid"; detail: string }
  | { type: "honba_invalid"; detail: string }
  | { type: "riichi_sticks_invalid"; detail: string }
  | { type: "ron_requires_discarder"; detail: string }
  | { type: "tsumo_cannot_have_discarder"; detail: string };

export type Participant =
  | "winner"
  | "opponent1"
  | "opponent2"
  | "opponent3"
  | "pot";

export type PaymentReason = "hand_points" | "honba" | "riichi_pot";

export type PaymentLine = {
  from: Participant;
  to: Participant;
  points: number;
  reason: PaymentReason;
};

export type CalcResult = {
  rule_set_id: string;
  breakdown: {
    base_points: number;
    limit: string;
    raw_base_points: number;
  };
  display_hand_points: string;
  payments: PaymentLine[];
  deltas: [Participant, number][];
  totals: {
    hand_points_total_to_winner: number;
    honba_points_total_to_winner: number;
    riichi_points_total_to_winner: number;
    total_to_winner: number;
  };
};

export type CalcResponse = {
  calc_version: string;
  result: CalcResult;
};

export type HandRecord = {
  id: string;
  user_id: string;
  created_at: string;
  rule_set_id: string;
  calc_version: string;
  fact: Fact;
  result: CalcResult;
  memo?: string | null;
  tags: string[];
};

export type HandSummary = {
  id: string;
  created_at: string;
  display_hand_points: string;
  total_to_winner: number;
  memo?: string | null;
  tags: string[];
};

export type RecalcResponse = {
  stored_calc_version: string;
  current_calc_version: string;
  same: boolean;
  stored_result: CalcResult;
  current_result: CalcResult;
};

