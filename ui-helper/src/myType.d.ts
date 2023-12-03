// export type UnionType = string | number;
export enum MinuteType {
  ByMinute = 0,
  ByHour,
  Duration,
  AppByMinute,
  AppByHour,
 }
 
export interface MinuteData {
  Minute: string;
  Distance: number;
  MouseCount: number;
  KeyCount: number;
  Date: string;
}

export interface MinuteAppData {
  Minute: string;
  Apps: number;
  MouseCount: number;
  KeyCount: number;
  Date: string;
}
export interface seriesData {
  name: string;
  value: number;
}