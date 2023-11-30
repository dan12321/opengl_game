#version 330 core
struct Material {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
    int shininess;
};

struct Light {
    vec3 position;
    vec3 diffuse;
    vec3 specular;
    float strength;
};

in vec2 TexCoord;
in vec3 FragPos;
in vec3 Normal;

uniform sampler2D texture0;
uniform vec3 lightColor;
uniform vec3 cameraPosition;
uniform Material material;
uniform Light light;

out vec4 FragColor;

void main()
{
    vec4 tex = texture2D(texture0, TexCoord);

    vec3 lightDir = normalize(light.position - FragPos);
    float lightDist = distance(FragPos, light.position);
    float distanceFallOff = min(1.0, light.strength / pow(lightDist, 2.0));

    float diffuseIntensity = max(dot(Normal, lightDir), 0.0);
    vec3 diffuseColor = distanceFallOff * diffuseIntensity * material.diffuse * light.diffuse;
    vec4 diffuse = vec4(diffuseColor, 1.0);

    vec3 cameraDir = normalize(cameraPosition - FragPos);
    vec3 reflectDir = reflect(-lightDir, Normal);
    float specularIntensity = distanceFallOff * pow(max(dot(cameraDir, reflectDir), 0.0), material.shininess);
    vec4 specular = vec4(specularIntensity * light.specular * material.specular, 1.0);

    vec4 ambient = vec4(material.ambient, 1.0);

    FragColor = (ambient + diffuse + specular) * tex;
}
