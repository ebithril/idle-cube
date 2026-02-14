#version 100
precision highp float;

uniform lowp vec3 CameraPos;

varying lowp vec3 vert_pos;
varying lowp vec3 vert_normal;
varying lowp vec4 color;

const vec3 lightPos = vec3(2.0, 2.0, 2.0);
const vec3 lightPos2 = vec3(-2.0, 2.0, 2.0);
const vec3 lightColor = vec3(1.0, 1.0, 1.0);
const float lightPower = 40.0;
const vec3 ambientColor = vec3(0.05, 0.05, 0.05);
const vec3 diffuseColor = vec3(0.5, 0.5, 0.5);
const vec3 specColor = vec3(1.0, 1.0, 1.0);
const float shininess = 16.0;

#ifndef PI
#define PI 3.14159265358
#endif

const float globalK0 = 0.00098;
const float globalK1 = 0.9921;
const float globalFakeLysMaxSpecularPower = (2.0 / (0.0014 * 0.0014)) - 2.0;
const float globalFMaxT = (exp2(-10.0 / sqrt((2.0 / (0.0014 * 0.0014)) - 2.0)) - 0.00098) / 0.9921;

float RangeAttenuation(vec3 aLightPos, float aLightRange, vec3 aWorldPosition)
{
	vec3 lightDir = aLightPos - aWorldPosition;
	float lengthToLight = length(lightDir);
	float b = lengthToLight / (aLightRange / 2.0);
	float attenuation = 1.0 - clamp(b, 0.0, 1.0);

	return attenuation;
}

vec3 Fresnel(const vec3 substance, const vec3 lightDir, const vec3 halfVec)
{
	vec3 oneMinusSubstance = 1.0 - substance;
	float LdotH = clamp(dot(lightDir, halfVec), 0.0, 1.0);
	float oneMinusLdotH = (1.0 - LdotH);
	float oneMinusLdotHPow5 = pow(oneMinusLdotH, 5.0);
	vec3 oneMinusSubstanceTimesOneMinusLdotHPow5 = oneMinusSubstance * oneMinusLdotHPow5;
	vec3 substancePlusOneMinusSubstanceTimesOneMinusLdotHPow5 = substance + oneMinusSubstanceTimesOneMinusLdotHPow5;
	return (substancePlusOneMinusSubstanceTimesOneMinusLdotHPow5);
}

vec3 ReflectionFresnel(const vec3 aSubstance, const vec3 aLightDirection, const vec3 aHalfVector, float aRoughness)
{
	return (aSubstance + (1.0 - aSubstance) * pow(1.0 - clamp(dot(aLightDirection, aHalfVector), 0.0, 1.0), 5.0) / (4.0 - 3.0 * aRoughness));
}

float RoughToSpecularPower(float aRoughness)
{
	return (2.0 / (aRoughness * aRoughness)) - 2.0;
}

float GetSpecularPowerToMip(float aSpecularPower, int aNumberOfMipsLevels)
{
	float fSmulMaxT = (exp2(-10.0 / sqrt(aSpecularPower)) - globalK0) / globalK1;

	return float(aNumberOfMipsLevels - 1) * (1.0 - clamp(fSmulMaxT / globalFMaxT, 0.0, 1.0));
}

float D_GGX(float aHalfVectorDotNormal, float aRoughness)
{
	float m = aRoughness * aRoughness;
	float m2 = m*m;
	float Denominator = aHalfVectorDotNormal * aHalfVectorDotNormal * (m2 - 1.0) + 1.0;
	float D = m2 / (PI * Denominator * Denominator);
	return D;
}

float V_Schlick_GGX(float aRoughness, float NdotV, float NdotL)
{
	float k = aRoughness * aRoughness * 0.5;
	float G1V = NdotV * (1.0 - k) + k;
	float G1L = NdotL * (1.0 - k) + k;
	return (0.25 / (G1V * G1L));
}

vec3 CalculateLight(vec3 albedo, float metalness, float roughness, float ao, vec3 pixelWorldPosition, vec3 worldNormal, vec3 aLightColor, vec3 aLightDirection,float anAttenuation)
{
	vec3 color = vec3(0.0,0.0,0.0);

	vec3 pixelMetalColor = albedo - (albedo * metalness);
	vec3 pixelNonMetalColor = (0.04 - 0.04 * metalness) + albedo * metalness;

	vec3 eyeDirection = normalize(pixelWorldPosition - CameraPos);
	float NdotV = clamp(dot(worldNormal, eyeDirection), 0.0, 1.0);

	vec3 negLightDir = -normalize(aLightDirection);
	vec3 halfVector = normalize(negLightDir - eyeDirection);

	float NdotL = clamp(dot(worldNormal, negLightDir), 0.0, 1.0);
	float HdotN = clamp(dot(halfVector, worldNormal), 0.0, 1.0);

	vec3 F = clamp(Fresnel(pixelNonMetalColor, negLightDir, halfVector), 0.0, 1.0);
	float D = clamp(D_GGX(HdotN, (roughness + 1.0) / 2.0), 0.0, 1.0);
	float V = clamp(V_Schlick_GGX((roughness + 1.0) / 2.0, NdotV, NdotL), 0.0, 1.0);

	vec3 spec = ((D * V * F) * NdotL);
	vec3 diffuse = (1.0 - F) * NdotL * pixelMetalColor * ao;

	color = ((spec + diffuse) / PI) * aLightColor;
	color *= anAttenuation * 3.0;
	return color;
}


vec3 GetSkyColor(vec3 direction, int mipLevel)
{
#if defined(PBL_AMBIENT_COLOR)
	return PBL_AMBIENT_COLOR;
//#elif defined(PBL_AMBIENT_USECUBEMAP)
	//return AmbientCubeMap.SampleLevel(DefaultSampler, direction, mipLevel).xyz;
#elif defined(PBL_TRILIGHT_SKY)
	float NdotL = dot(direction, vec3(0, 1, 0));// * rsqrt(dot(pixelNormal, pixelNormal));
	vec3 trilight = 0;
		trilight += saturate(NdotL) * PBL_TRILIGHT_SKY;
	trilight += (1 - abs(NdotL)) * PBL_TRILIGHT_EQU;
	trilight += saturate(-NdotL) * PBL_TRILIGHT_GRN;

	vec3 blurredTri = saturate((PBL_TRILIGHT_SKY + PBL_TRILIGHT_EQU + PBL_TRILIGHT_GRN) / 3.f);

	return lerp(trilight, blurredTri, mipLevel / PBL_MAX_MIPMAP);
#else
	return vec3(0, 0, 0);
#endif
}

vec3 GetReflectedSkyColor(vec3 pixelPosition, vec3 pixelNormal, int mipLevel, vec3 aCameraPosition)
{
	vec3 eyeDir = normalize(pixelPosition - aCameraPosition);
	vec3 reflectionVector = reflect(eyeDir, pixelNormal);

	return GetSkyColor(reflectionVector, mipLevel);
}

vec3 CalculateAmbientColor(vec3 albedo, float metalness, float roughness, float ao, vec3 pixelWorldPosition, vec3 worldNormal)
{
	vec3 pixelMetalColor = albedo - (albedo * metalness);
	vec3 pixelNonMetalColor = (0.04 - 0.04 * metalness) + albedo * metalness;

	vec3 eyeDirection = normalize(pixelWorldPosition - CameraPos).xyz;
	vec3 reflectionFresnel = ReflectionFresnel(pixelNonMetalColor, worldNormal, -eyeDirection, 1.0 - pow(8192.0, roughness)).xyz;

	vec3 reflectedSkyColor = GetReflectedSkyColor(pixelWorldPosition.xyz, worldNormal, 7, CameraPos);
	vec3 ambientDiffuse = (reflectedSkyColor * (1.0 - reflectionFresnel) * ao * pixelMetalColor).xyz;

	float fakeLysSpecularPower = RoughToSpecularPower(roughness);
	float lysMipMap = GetSpecularPowerToMip(fakeLysSpecularPower, 12);

	vec3 ambientSpecular = (GetReflectedSkyColor(pixelWorldPosition.xyz, worldNormal, int(lysMipMap), CameraPos).xyz * ao * (reflectionFresnel));//* (1 - roughness)).xyz;
	vec3 ambientLighting = ambientDiffuse + ambientSpecular;

	return ambientLighting;
}

vec3 NearestPointOnFiniteLine(vec3 start, vec3 end, vec3 pnt)
{
	vec3 line = end - start;
	float len = length(line);
	line = line / len;
	vec3 v = pnt - start;
	float d = dot(v, line);
	d = clamp(d, 0.0, len);
	return start + line * d;
}

vec3 CalcLightForPoint(vec3 LightPos, float LightRange, vec3 albedo, float metalness, float roughness, float ao, vec3 pixelWorldPosition, vec3 worldNormal, vec3 aLightColor) {
	float attenuation = RangeAttenuation(LightPos, LightRange, pixelWorldPosition);
	return CalculateLight(albedo, metalness, roughness, ao, pixelWorldPosition, worldNormal, lightColor, normalize(pixelWorldPosition - LightPos), attenuation);
}

void main(){
	vec3 ViewDir = normalize(vert_pos - CameraPos);
	vec3 normal = normalize(vert_normal);
	float metalness = 0.0;
	float roughness = 1.0;
	float ao = 1.0;

	vec3 ambientLight = CalculateAmbientColor(color.xyz, metalness, roughness, ao, vert_pos, normal);

	vec3 lightPoint = NearestPointOnFiniteLine(lightPos, lightPos2, vert_pos);

	float LightRange = 10.0;

	vec3 pbr_color = ambientLight + CalcLightForPoint(lightPoint, LightRange, color.xyz, metalness, roughness, ao, vert_pos, normal, vec3(2.0,2.0, 2.0));
	pbr_color += CalcLightForPoint(lightPos, LightRange, color.xyz, metalness, roughness, ao, vert_pos, normal, vec3(2.0,2.0, 2.0));
	pbr_color += CalcLightForPoint(lightPos2, LightRange, color.xyz, metalness, roughness, ao, vert_pos, normal, vec3(2.0,2.0, 2.0));

	gl_FragColor = vec4(pbr_color, 1.0);
}
