export type TranslationDict = {
  navProducts: string;
  navDocuments: string;
  navIllustrations: string;
  navPartners: string;
  navContact: string;
  badge: string;
  titleMain: string;
  titleSub: string;
  desc: string;
  btnRegister: string;
  btnGetStarted: string;
  statClients: string;
  statCountries: string;
  statWebsites: string;
  statAttacks: string;
  terminalTitle: string;
  terminalSub: string;
  liveFeedTitle: string;
  liveFeedDesc: string;
  registerSuccess: string;
  provisionTitle: string;
  provisionDesc: string;
};

// A settlement-pipeline event in the live stream.
export interface ThreatLog {
  id: string;
  time: string;
  type: "release" | "proof" | "verify" | "fund" | "reject";
  target: string;
  status: string;
  country: string;
}
